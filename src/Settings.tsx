import { useMemo, useState, useEffect } from "react";
import {
  OffsetsInterface,
  SettingsInterface,
  getStoredSettings,
  makeDefaultSettings,
  offsetsValidationSchema,
  updateStoredSettings,
} from "./helpers";
import { invoke } from "@tauri-apps/api/tauri";
import { emit, listen } from "@tauri-apps/api/event";
import { z } from "zod";

const Settings = () => {
  const [hasStarted, setHasStarted] = useState(false);
  const [settings, setSettings] = useState<SettingsInterface>(
    makeDefaultSettings()
  );
  const [offsets, setOffsets] = useState<OffsetsInterface>(
    {} as OffsetsInterface
  );
  const [offsetsInput, setOffsetsInput] = useState("");

  const configError = useMemo(() => {
    try {
      const data = JSON.parse(offsetsInput);
      const res = offsetsValidationSchema.safeParse(data);
      if (res.success) {
        return null;
      }

      const errors = res.error.flatten().fieldErrors;
      const errorKeys = Object.keys(errors);
      const errorMessages = errorKeys.map((k: any) => {
        const temp_error = (errors as Record<string, string>)[k];
        return `${k}: ${temp_error}`;
      });
      return errorMessages.join("\n");
    } catch (e) {
      return "Invalid Config";
    }
  }, [offsetsInput]);

  const invoteStart = async () => {
    await invoke("start", {
      data: offsets,
    });
  };

  const invoteStop = async () => {
    await invoke("stop");
  };

  useEffect(() => {
    emit("settings_changed", settings);
  }, [settings]);

  const listenAppStateChange = async () => {
    await listen("app_state_change", (data) => {
      setHasStarted(data.payload as boolean);
    });
  };

  useEffect(() => {
    listenAppStateChange();
  }, []);

  useEffect(() => {
    if (configError) {
      return;
    }

    const data = JSON.parse(offsetsInput);
    setOffsets(data);
    updateStoredSettings({
      offsets: data,
      settings: settings,
    });
  }, [offsetsInput, configError, settings]);

  useEffect(() => {
    const res = getStoredSettings();
    setSettings(res.settings);
    setOffsets(res.offsets);
    setOffsetsInput(JSON.stringify(res.offsets, null, 2));
  }, []);

  const settingsArr = useMemo((): {
    objKey: keyof SettingsInterface;
    label: string;
  }[] => {
    return [
      {
        label: "Show Boxes",
        objKey: "showBoxes",
      },
      {
        label: "Show Bones",
        objKey: "showBones",
      },
      {
        label: "Show Health",
        objKey: "showHealth",
      },
      {
        label: "Show Weapon Name",
        objKey: "showWeapon",
      },
      {
        label: "Show Crosshair",
        objKey: "showCrosshair",
      },
      {
        label: "Show Teammates",
        objKey: "showTeammates",
      },
      {
        label: "Show Enemies",
        objKey: "showEnemies",
      },
    ];
  }, []);

  return (
    <div className={"settings-page"}>
      <h1>Settings</h1>
      {!hasStarted && (
        <button className="action-button" onClick={invoteStart}>
          Start
        </button>
      )}
      {hasStarted && (
        <button className="action-button" onClick={invoteStop}>
          Stop
        </button>
      )}

      <div className="checkbox-area">
        {settingsArr.map((s) => {
          return (
            <div>
              <label htmlFor={s.label}>{s.label}</label>

              <input
                id={s.label}
                type="checkbox"
                checked={settings[s.objKey]}
                onChange={(e) =>
                  setSettings((prev) => ({
                    ...prev,
                    [s.objKey]: e.target.checked,
                  }))
                }
              />
            </div>
          );
        })}
      </div>

      <div className="config-area">
        <textarea
          value={offsetsInput}
          onChange={(e) => setOffsetsInput(e.target.value)}
        />
        {configError && <pre className="errors">{configError}</pre>}
      </div>
    </div>
  );
};

export default Settings;
