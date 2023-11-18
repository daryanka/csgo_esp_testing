import { useMemo, useState, useEffect } from "react";
import { SettingsInterface, makeDefaultSettings } from "./helpers";
import { invoke } from "@tauri-apps/api/tauri";
import { emit, listen } from "@tauri-apps/api/event";

const Settings = () => {
  const [hasStarted, setHasStarted] = useState(false);
  const [settings, setSettings] = useState<SettingsInterface>(
    makeDefaultSettings()
  );

  const invoteStart = async () => {
    await invoke("start");
  };

  const invoteStop = async () => {
    await invoke("stop");
  };

  useEffect(() => {
    emit("settings_changed", settings);
  }, [settings]);

  const listenAppStateChange = async () => {
    await listen("app_state_change", (data) => {
      console.log("here", data.payload);
      setHasStarted(data.payload as boolean);
    });
  };

  useEffect(() => {
    listenAppStateChange();
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
      <h1>settings</h1>
      {!hasStarted && <button onClick={invoteStart}>Start</button>}
      {hasStarted && <button onClick={invoteStop}>Stop</button>}

      <form>
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
      </form>
    </div>
  );
};

export default Settings;
