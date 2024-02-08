import { z } from "zod";

export interface OffsetsInterface {
  width: number;
  height: number;
  dwEntityList: number;
  dwLocalPlayerController: number;
  dwViewMatrix: number;
  m_pClippingWeapon: number;
  m_iHealth: number;
  m_iTeamNum: number;
  m_hPlayerPawn: number;
  m_vecAbsOrigin: number;
  m_vOldOrigin: number;
}

export interface LocalStorageInterface {
  offsets: OffsetsInterface;
  settings: SettingsInterface;
}

export const offsetsValidationSchema = z.object({
  width: z.number().positive(),
  height: z.number().positive(),
  dwEntityList: z.number().positive(),
  dwLocalPlayerController: z.number().positive(),
  dwViewMatrix: z.number().positive(),
  m_pClippingWeapon: z.number().positive(),
  m_iHealth: z.number().positive(),
  m_iTeamNum: z.number().positive(),
  m_hPlayerPawn: z.number().positive(),
  m_vecAbsOrigin: z.number().positive(),
  m_vOldOrigin: z.number().positive(),
});

export const getStoredSettings = (): LocalStorageInterface => {
  const res = localStorage.getItem("settings");
  if (res) {
    return JSON.parse(res);
  }

  const initial = {
    offsets: {
      width: 0,
      height: 0,
      dwEntityList: 0,
      dwLocalPlayerController: 0,
      dwViewMatrix: 0,
      m_pClippingWeapon: 0,
      m_iHealth: 0,
      m_iTeamNum: 0,
      m_hPlayerPawn: 0,
      m_vecAbsOrigin: 0,
      m_vOldOrigin: 0,
    },
    settings: makeDefaultSettings(),
  };

  localStorage.setItem("settings", JSON.stringify(initial));

  return initial;
};

export const updateStoredSettings = (settings: LocalStorageInterface) => {
  localStorage.setItem("settings", JSON.stringify(settings));
};

export interface SettingsInterface {
  showBoxes: boolean;
  showBones: boolean;
  showHealth: boolean;
  showWeapon: boolean;
  showCrosshair: boolean;
  showTeammates: boolean;
  showEnemies: boolean;
  opacity: number;
}

export const makeDefaultSettings = (): SettingsInterface => ({
  showBones: true,
  showHealth: true,
  showWeapon: true,
  showBoxes: true,
  showCrosshair: true,
  showTeammates: true,
  showEnemies: true,
  opacity: 0.3,
});
