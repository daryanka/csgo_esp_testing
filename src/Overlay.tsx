import { listen } from "@tauri-apps/api/event";
import { useEffect, useMemo, useState } from "react";
import { useWindowSize } from "usehooks-ts";
import _ from "lodash";
import { SettingsInterface, makeDefaultSettings } from "./helpers";

interface Player {
  id: string;
  x: number;
  y: number;
  isEnemy: boolean;
  width: number;
  height: number;
  health: number;
  weapon_name: string;
  head_radius: number;
  bones: {
    bone_kind:
      | "head"
      | "neck_0"
      | "spine_1"
      | "spine_2"
      | "pelvis"
      | "arm_upper_L"
      | "arm_lower_L"
      | "hand_L"
      | "arm_upper_R"
      | "arm_lower_R"
      | "hand_R"
      | "leg_upper_L"
      | "leg_lower_L"
      | "ankle_L"
      | "leg_upper_R"
      | "leg_lower_R"
      | "ankle_R";
    x: number;
    y: number;
  }[];
}

interface GameData {
  players: Player[];
}

const GREEN = "#00ff00";
const RED = "#ff0000";

function Overlay() {
  const { width: windowWidth, height: windowHeight } = useWindowSize();
  const [settings, setSettings] = useState<SettingsInterface>(
    makeDefaultSettings()
  );
  const [rawData, setData] = useState<GameData>({
    players: [],
  });

  const listenerToPlayerData = async () => {
    await listen("player_data", (data) => {
      setData(data.payload as GameData);
    });
  };

  const listenToSettings = async () => {
    await listen("settings_changed", (data) => {
      setSettings(data.payload as SettingsInterface);
    });
  };

  useEffect(() => {
    listenerToPlayerData();
    listenToSettings();
  }, []);

  const data = useMemo((): GameData => {
    return {
      players: rawData.players.filter((player) => {
        if (player.isEnemy && settings.showEnemies) {
          return true;
        }
        if (!player.isEnemy && settings.showTeammates) {
          return true;
        }
        return false;
      }),
    };
  }, [settings, rawData]);

  return (
    <div className={"overlay-page"}>
      {data.players.map((player) => {
        return (
          <Box key={`box-${player.id}`} player={player} settings={settings} />
        );
      })}
      {settings.showBones && (
        <svg
          className={"app-svg"}
          viewBox={`0 0 ${windowWidth} ${windowHeight}`}
        >
          {data.players.map((player) => {
            return <Skeleton key={`skeleton-${player.id}`} player={player} />;
          })}
        </svg>
      )}
      {settings.showCrosshair && (
        <div
          className="cross-hair"
          style={{ top: windowHeight / 2, left: windowWidth / 2 }}
        />
      )}
    </div>
  );
}

const Skeleton = ({ player }: { player: Player }) => {
  const { bones } = player;

  const path = useMemo(() => {
    if (bones.length === 0) {
      return "";
    }

    let path = "";
    const {
      head,
      neck_0,
      spine_1,
      spine_2,
      pelvis,
      leg_upper_L,
      leg_lower_L,
      ankle_L,
      leg_upper_R,
      leg_lower_R,
      arm_lower_L,
      arm_lower_R,
      arm_upper_L,
      hand_L,
      hand_R,
      arm_upper_R,
      ankle_R,
    } = _.keyBy(bones, "bone_kind") as Record<
      Player["bones"][number]["bone_kind"],
      Player["bones"][number]
    >;

    path += `M ${head.x} ${head.y} `;
    path += `L ${neck_0.x} ${neck_0.y} `;
    path += `L ${spine_1.x} ${spine_1.y} `;
    path += `L ${spine_2.x} ${spine_2.y} `;
    path += `L ${pelvis.x} ${pelvis.y} `;
    path += `L ${leg_upper_L.x} ${leg_upper_L.y} `;
    path += `L ${leg_lower_L.x} ${leg_lower_L.y} `;
    path += `L ${ankle_L.x} ${ankle_L.y} `;
    path += `M ${pelvis.x} ${pelvis.y} `;
    path += `L ${leg_upper_R.x} ${leg_upper_R.y} `;
    path += `L ${leg_lower_R.x} ${leg_lower_R.y} `;
    path += `L ${ankle_R.x} ${ankle_R.y} `;
    path += `M ${spine_1.x} ${spine_1.y} `;
    path += `L ${arm_upper_L.x} ${arm_upper_L.y} `;
    path += `L ${arm_lower_L.x} ${arm_lower_L.y} `;
    path += `L ${hand_L.x} ${hand_L.y} `;
    path += `M ${spine_1.x} ${spine_1.y} `;
    path += `L ${arm_upper_R.x} ${arm_upper_R.y} `;
    path += `L ${arm_lower_R.x} ${arm_lower_R.y} `;
    path += `L ${hand_R.x} ${hand_R.y} `;

    return path;
  }, [player]);

  if (bones.length === 0) {
    return null;
  }

  const head = bones.find((bone) => bone.bone_kind == "head");

  return (
    <g>
      {/* Draw head circle*/}
      {head && (
        <circle
          cx={head.x}
          cy={head.y}
          r={player.head_radius}
          stroke={player.isEnemy ? RED : GREEN}
          strokeWidth={4}
          fill={"transparent"}
        />
      )}
      <path stroke={player.isEnemy ? RED : GREEN} strokeWidth={4} d={path} />
    </g>
  );
};

const Box = ({
  player,
  settings,
}: {
  player: Player;
  settings: SettingsInterface;
}) => {
  const { health, height, isEnemy, weapon_name, width, x, y } = player;
  return (
    <>
      {settings.showBoxes && (
        <div
          className={"box"}
          style={{
            top: y,
            left: x,
            height: height,
            width: width,
            borderColor: isEnemy ? RED : GREEN,
          }}
        />
      )}
      <p
        className={"health"}
        style={{
          top: y - 45,
          left: x,
          color: isEnemy ? RED : GREEN,
        }}
      >
        {settings.showHealth && <>Health: {health}</>}
        {settings.showHealth && settings.showWeapon && (
          <>
            <br />
          </>
        )}
        {settings.showWeapon && <>Weapon: {weapon_name}</>}
      </p>
    </>
  );
};

export default Overlay;
