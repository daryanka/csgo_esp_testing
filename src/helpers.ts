export interface SettingsInterface {
  showBoxes: boolean;
  showBones: boolean;
  showHealth: boolean;
  showWeapon: boolean;
  showCrosshair: boolean;
  showTeammates: boolean;
  showEnemies: boolean;
}

export const makeDefaultSettings = (): SettingsInterface => ({
  showBones: true,
  showHealth: true,
  showWeapon: true,
  showBoxes: true,
  showCrosshair: true,
  showTeammates: true,
  showEnemies: true,
});
