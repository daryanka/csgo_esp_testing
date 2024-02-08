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

      <LoadOffsetsButton setOffsets={setOffsetsInput} />

      <div className="checkbox-area">
        {settingsArr.map((s) => {
          return (
            <div key={s.label}>
              <label htmlFor={s.label}>{s.label}</label>

              <input
                id={s.label}
                type="checkbox"
                checked={settings[s.objKey] as boolean}
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

      <label>Opacity</label>
      <input
        type="range"
        min="1"
        max="100"
        value={settings.opacity * 100}
        id="myRange"
        onChange={(e) => {
          const newOpp = e.target.valueAsNumber / 100;
          setSettings((prev) => ({
            ...prev,
            opacity: newOpp,
          }));
        }}
      />

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

const LoadOffsetsButton = ({
  setOffsets,
}: {
  setOffsets: React.Dispatch<React.SetStateAction<string>>;
}) => {
  const [isLoading, setIsLoading] = useState(false);

  const generate_offsets = async () => {
    if (isLoading) {
      return;
    }
    setIsLoading(true);
    try {
      const res =
        await invoke<Omit<OffsetsInterface, "height" | "width">>(
          "generate_offsets"
        );
      setOffsets((prev) => {
        try {
          const old = JSON.parse(prev);
          const newData = {
            ...old,
            ...res,
          };
          return JSON.stringify(newData, null, 2);
        } catch (e) {
          return JSON.stringify(res, null, 2);
        }
      });
      setIsLoading(false);
    } catch (e) {
      setIsLoading(false);
    }
  };

  return (
    <div>
      <button onClick={generate_offsets}>
        {isLoading ? "Loading, this may take 20 seconds" : "Load Offsets"}
      </button>
    </div>
  );
};

export default Settings;
