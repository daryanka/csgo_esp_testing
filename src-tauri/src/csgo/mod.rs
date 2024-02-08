mod offsets;
mod process;

use tauri::{AppHandle, Manager};

use self::process::{read_process_memory, InternalProcess};
use crate::{
    csgo::process::{get_base_module_address, get_process_handle},
    AppStateType,
};
use nalgebra::{Matrix4, Vector2, Vector3};

const CLIENT_DLL: &str = "client.dll";

#[derive(serde::Serialize, serde::Deserialize, Default, Clone)]
struct PlayerDataTransfer {
    id: String,
    x: f32,
    y: f32,
    width: f32,
    height: f32,
    #[serde(rename = "isEnemy")]
    is_enemy: bool,
    health: u32,
    bones: Vec<BonePositions>,
    head_radius: f32,
    weapon_name: String,
}

async fn stop_all(app_state: AppStateType, app_handle: AppHandle) {
    let mut state = app_state.0.write().await;
    state.is_reading_memory = false;
    let _ = app_handle.emit_all("app_state_change", false);
    let _ = app_handle.emit_all("player_data", GameDataTransfer { players: vec![] });
}

pub async fn start(
    app_state: AppStateType,
    app_handle: AppHandle,
    screen_size: ScreenAndDynamicOffsets,
) {
    let _ = app_handle.emit_all("app_state_change", true);

    let proc = get_process_handle("cs2.exe");
    if let None = proc {
        stop_all(app_state, app_handle).await;
        return;
    }

    let mut proc = proc.unwrap();
    println!("Proc is not empty: {:?}", proc.name);

    if let Err(e) = get_base_module_address(&mut proc, CLIENT_DLL) {
        println!("Error getting client module: {}", e);
        stop_all(app_state, app_handle).await;
        return;
    }

    println!("{:#?}", proc);

    loop {
        std::thread::sleep(std::time::Duration::from_millis(5));

        {
            let state = app_state.0.read().await;
            if !state.is_reading_memory {
                let _ = app_handle.emit_all("app_state_change", false);
                let _ = app_handle.emit_all("player_data", GameDataTransfer { players: vec![] });
                return;
            }
        }

        let view_matrix = get_view_matrix(&proc, &screen_size);
        let players = get_players(&proc, &screen_size);
        let view_matrix = match view_matrix {
            Some(matrix) => matrix,
            None => continue,
        };

        let boxes = players_to_boxes(players, view_matrix, screen_size.width, screen_size.height);

        let data = GameDataTransfer { players: boxes };
        let _ = app_handle.emit_all("player_data", data);
        continue;
    }
}
// const ENGINE_DLL: &str = "engine.dll";
#[derive(serde::Serialize, serde::Deserialize, Default, Clone)]
pub struct GameDataTransfer {
    players: Vec<PlayerDataTransfer>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ScreenAndDynamicOffsets {
    pub width: usize,
    pub height: usize,
    pub dwEntityList: usize,
    pub dwLocalPlayerController: usize,
    pub dwViewMatrix: usize,
    pub m_iHealth: usize,
    pub m_iTeamNum: usize,
    pub m_hPlayerPawn: usize,
    pub m_vecAbsOrigin: usize,
    pub m_vOldOrigin: usize,
    pub m_pClippingWeapon: usize,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, Default)]
pub struct DynamicOffsets {
    pub dwEntityList: usize,
    pub dwLocalPlayerController: usize,
    pub dwViewMatrix: usize,
    pub m_iHealth: usize,
    pub m_iTeamNum: usize,
    pub m_hPlayerPawn: usize,
    pub m_vecAbsOrigin: usize,
    pub m_vOldOrigin: usize,
    pub m_pClippingWeapon: usize,
}

#[derive(Debug, Clone)]
struct Player {
    health: u32,
    team_number: u8,
    player_pawn: usize,
    controller_base: usize,
    list_entry: usize,
    list_entry2: usize,
    cs_player_pawn: usize,
    is_teammate: bool,
    is_local_player: bool,
    origin_vec3: Vector3<f32>,
    bones_list: Option<Vec<Bone>>,
    weapon_name: String,
}

fn get_players(proc: &InternalProcess, offsets_config: &ScreenAndDynamicOffsets) -> Vec<Player> {
    let max_players = 64;

    let mut entity_list: usize = 0;
    read_process_memory(
        proc,
        &mut entity_list,
        proc.get_module_base(CLIENT_DLL) + (offsets_config.dwEntityList as usize),
    );
    let mut all_players = vec![];

    // Try get local player address
    let mut local_player_addr: usize = 0;
    let _ = read_process_memory(
        proc,
        &mut local_player_addr,
        proc.get_module_base(CLIENT_DLL) + offsets_config.dwLocalPlayerController as usize,
    );

    // loop through all players
    for player_index in 0..max_players {
        let player = get_player(
            proc,
            entity_list,
            player_index,
            local_player_addr,
            offsets_config,
        );
        if let Some(player) = player {
            all_players.push(player);
        }
    }

    // Add is teammate or not
    let local_player = all_players.iter().find(|&p| p.is_local_player).cloned();

    if let Some(local_player) = local_player {
        for player in all_players.iter_mut() {
            if player.team_number == local_player.team_number {
                player.is_teammate = true;
            }
        }
    }

    return all_players;
}

fn get_player(
    proc: &InternalProcess,
    entity_list: usize,
    player_index: usize,
    local_player_addr: usize,
    offsets_config: &ScreenAndDynamicOffsets,
) -> Option<Player> {
    let mut list_entry: usize = 0;
    let _ = read_process_memory(proc, &mut list_entry, entity_list + 0x10);
    if list_entry == 0 {
        return None;
    }

    let mut controller_base: usize = 0;
    let _ = read_process_memory(
        proc,
        &mut controller_base,
        list_entry + offsets::ENTITY_SPACEING * (player_index & 0x1FF),
    );
    if controller_base == 0 {
        return None;
    }

    let mut player_pawn: usize = 0;
    let _ = read_process_memory(
        proc,
        &mut player_pawn,
        controller_base + offsets_config.m_hPlayerPawn,
    );

    if player_pawn == 0 {
        return None;
    }

    let mut list_entry2: usize = 0;
    let _ = read_process_memory(
        proc,
        &mut list_entry2,
        (entity_list as usize) + 0x8 * ((player_pawn & 0x7FFF) >> 9) + 16,
    );

    if list_entry2 == 0 {
        return None;
    }

    let mut cs_player_pawn: usize = 0;
    let _ = read_process_memory(
        proc,
        &mut cs_player_pawn,
        list_entry2 + offsets::ENTITY_SPACEING * (player_pawn & 0x1FF),
    );

    if cs_player_pawn == 0 {
        return None;
    }

    let mut health: u32 = 0;
    let _ = read_process_memory(proc, &mut health, cs_player_pawn + offsets_config.m_iHealth);

    let mut team_number: u8 = 0;
    let _ = read_process_memory(
        proc,
        &mut team_number,
        cs_player_pawn + offsets_config.m_iTeamNum,
    );

    // Get origin
    let mut origin: Vec<f32> = vec![0.0, 0.0, 0.0];
    let _ = read_process_memory(
        proc,
        &mut origin[0],
        cs_player_pawn + offsets_config.m_vOldOrigin,
    );
    let _ = read_process_memory(
        proc,
        &mut origin[1],
        cs_player_pawn + offsets_config.m_vOldOrigin + 4,
    );
    let _ = read_process_memory(
        proc,
        &mut origin[2],
        cs_player_pawn + offsets_config.m_vOldOrigin + 8,
    );

    // origin as matrix
    let matrix: Vector3<f32> = Vector3::new(origin[0], origin[1], origin[2]);

    let bones = get_bones(proc, cs_player_pawn);

    let weapon_name = get_weapon(proc, cs_player_pawn, &offsets_config);

    Some(Player {
        health,
        team_number,
        player_pawn,
        controller_base,
        list_entry,
        list_entry2,
        cs_player_pawn,
        is_local_player: local_player_addr == controller_base,
        is_teammate: false,
        origin_vec3: matrix,
        bones_list: bones,
        weapon_name: weapon_name,
    })
}

fn get_weapon(
    proc: &InternalProcess,
    player_pawn: usize,
    offsets_config: &ScreenAndDynamicOffsets,
) -> String {
    let mut weapon_addr: usize = 0;
    let _ = read_process_memory(
        proc,
        &mut weapon_addr,
        player_pawn + offsets_config.m_pClippingWeapon,
    );
    if weapon_addr == 0 {
        return "".to_string();
    }

    let mut weapon_data: usize = 0;
    let _ = read_process_memory(proc, &mut weapon_data, weapon_addr + 0x360);
    if weapon_data == 0 {
        return "".to_string();
    }

    let mut name_ptr: usize = 0;
    let _ = read_process_memory(proc, &mut name_ptr, weapon_data + 0xc18);
    if name_ptr == 0 {
        return "".to_string();
    }

    let mut raw_name = [0u8; 32];
    let _ = read_process_memory(proc, &mut raw_name, name_ptr);

    let name = String::from_utf8(raw_name.into()).unwrap_or("".to_owned());
    let name = name.trim();
    let name = name.split("\0").collect::<Vec<&str>>()[0].replace("weapon_", "");
    name
}

#[derive(Debug, Clone, PartialEq)]
enum BoneKind {
    head,
    neck_0,
    spine_1,
    spine_2,
    pelvis,
    arm_upper_L,
    arm_lower_L,
    hand_L,
    arm_upper_R,
    arm_lower_R,
    hand_R,
    leg_upper_L,
    leg_lower_L,
    ankle_L,
    leg_upper_R,
    leg_lower_R,
    ankle_R,
}

impl BoneKind {
    fn get_index(&self) -> usize {
        match self {
            Self::head => 6,
            Self::neck_0 => 5,
            Self::spine_1 => 4,
            Self::spine_2 => 2,
            Self::pelvis => 0,
            Self::arm_upper_L => 8,
            Self::arm_lower_L => 9,
            Self::hand_L => 10,
            Self::arm_upper_R => 13,
            Self::arm_lower_R => 14,
            Self::hand_R => 15,
            Self::leg_upper_L => 22,
            Self::leg_lower_L => 23,
            Self::ankle_L => 24,
            Self::leg_upper_R => 25,
            Self::leg_lower_R => 26,
            Self::ankle_R => 27,
        }
    }

    fn to_str(&self) -> &str {
        match self {
            Self::head => "head",
            Self::neck_0 => "neck_0",
            Self::spine_1 => "spine_1",
            Self::spine_2 => "spine_2",
            Self::pelvis => "pelvis",
            Self::arm_upper_L => "arm_upper_L",
            Self::arm_lower_L => "arm_lower_L",
            Self::hand_L => "hand_L",
            Self::arm_upper_R => "arm_upper_R",
            Self::arm_lower_R => "arm_lower_R",
            Self::hand_R => "hand_R",
            Self::leg_upper_L => "leg_upper_L",
            Self::leg_lower_L => "leg_lower_L",
            Self::ankle_L => "ankle_L",
            Self::leg_upper_R => "leg_upper_R",
            Self::leg_lower_R => "leg_lower_R",
            Self::ankle_R => "ankle_R",
        }
    }
}

#[derive(Debug, Clone)]
struct Bone {
    kind: BoneKind,
    matrix: Vector3<f32>,
}

fn get_bones(proc: &InternalProcess, player_pawn: usize) -> Option<Vec<Bone>> {
    let all_bones: Vec<BoneKind> = vec![
        BoneKind::head,
        BoneKind::neck_0,
        BoneKind::spine_1,
        BoneKind::spine_2,
        BoneKind::pelvis,
        BoneKind::arm_upper_L,
        BoneKind::arm_lower_L,
        BoneKind::hand_L,
        BoneKind::arm_upper_R,
        BoneKind::arm_lower_R,
        BoneKind::hand_R,
        BoneKind::leg_upper_L,
        BoneKind::leg_lower_L,
        BoneKind::ankle_L,
        BoneKind::leg_upper_R,
        BoneKind::leg_lower_R,
        BoneKind::ankle_R,
    ];

    let mut game_scene_node: usize = 0;
    let _ = read_process_memory(
        proc,
        &mut game_scene_node,
        player_pawn + offsets::PAWN_GAME_SCENE_NODE,
    );
    if game_scene_node == 0 {
        return None;
    }
    let mut bone_array_addr: usize = 0;
    let _ = read_process_memory(
        proc,
        &mut bone_array_addr,
        game_scene_node + offsets::MODEL_STATE + offsets::BONE_ARRAY,
    );
    if bone_array_addr == 0 {
        return None;
    }

    let mut results = vec![];

    for bone in all_bones.iter() {
        let mut x: f32 = 0.0;
        let mut y: f32 = 0.0;
        let mut z: f32 = 0.0;
        let bone_start_addr: usize = bone_array_addr + (bone.get_index() * 0x20); // 0x20 = 32 for bone spacing
        let _ = read_process_memory(proc, &mut x, bone_start_addr);
        let _ = read_process_memory(proc, &mut y, bone_start_addr + 4);
        let _ = read_process_memory(proc, &mut z, bone_start_addr + 8);

        if x == 0.0 && y == 0.0 && z == 0.0 {
            continue;
        }

        let bone_matrix: Vector3<f32> = Vector3::new(x, y, z);
        results.push(Bone {
            kind: bone.clone(),
            matrix: bone_matrix,
        });
    }

    if results.len() != all_bones.len() {
        return None;
    }

    Some(results)
}

fn get_view_matrix(
    proc: &InternalProcess,
    offsets_config: &ScreenAndDynamicOffsets,
) -> Option<Matrix4<f32>> {
    // 2d array of floats
    // [
    //     [0.0, 0.0, 0.0, 0.0],
    //     [0.0, 0.0, 0.0, 0.0],
    //     [0.0, 0.0, 0.0, 0.0],
    //     [0.0, 0.0, 0.0, 0.0],
    // ]
    let mut view_matrix = Vec::<Vec<f32>>::new();

    let mut offset = proc.get_module_base(CLIENT_DLL) + offsets_config.dwViewMatrix;

    for _ in 0..4 {
        let mut curr = Vec::<f32>::new();

        for _ in 0..4 {
            let mut curr_float: f32 = 0.0;
            let _ = read_process_memory(proc, &mut curr_float, offset);
            curr.push(curr_float);
            offset += 4; // 4 bytes per float
        }

        view_matrix.push(curr);
    }
    // verify is 4x4
    if view_matrix.len() != 4 {
        return None;
    }
    for row in view_matrix.iter() {
        if row.len() != 4 {
            return None;
        }
    }

    let new_matrix = Matrix4::new(
        view_matrix[0][0],
        view_matrix[0][1],
        view_matrix[0][2],
        view_matrix[0][3],
        view_matrix[1][0],
        view_matrix[1][1],
        view_matrix[1][2],
        view_matrix[1][3],
        view_matrix[2][0],
        view_matrix[2][1],
        view_matrix[2][2],
        view_matrix[2][3],
        view_matrix[3][0],
        view_matrix[3][1],
        view_matrix[3][2],
        view_matrix[3][3],
    );
    Some(new_matrix)
}

fn players_to_boxes(
    players: Vec<Player>,
    view_matrix: Matrix4<f32>,
    screen_width: usize,
    screen_height: usize,
) -> Vec<PlayerDataTransfer> {
    let mut data = vec![];
    for player in players {
        if player.is_local_player {
            continue;
        }

        if player.health == 0 {
            continue;
        }

        let window_location = Vector2::new(0.0, 0.0);

        let pos = player.origin_vec3;
        let view_offset: Vector3<f32> = Vector3::new(0.0, 0.0, 70.0);
        let abs = pos + view_offset;

        let origin_screen_pos =
            world_to_screen(view_matrix, pos, screen_width, screen_height, false);
        if origin_screen_pos.is_none() {
            continue;
        }

        // Origin
        let origin_screen_pos = origin_screen_pos.unwrap() + window_location;

        let abs_screen_position = world_to_screen(
            view_matrix,
            abs,
            screen_width as usize,
            screen_height as usize,
            false,
        );
        if abs_screen_position.is_none() {
            continue;
        }
        let abs_screen_position = abs_screen_position.unwrap() + window_location;

        let box_width: Vector2<f32> =
            Vector2::new((origin_screen_pos.y - abs_screen_position.y) / 2.0, 0.0);
        let box_start = abs_screen_position - box_width;
        let box_end = origin_screen_pos + box_width;

        let (bones, head_radius) = get_bones_pos(&player, view_matrix, screen_width, screen_height)
            .unwrap_or((vec![], 0.0));

        let transfer = PlayerDataTransfer {
            id: player.player_pawn.to_string(),
            is_enemy: !player.is_teammate,
            x: box_start.x,
            y: box_start.y,
            width: box_end.x - box_start.x,
            height: box_end.y - box_start.y,
            health: player.health,
            bones: bones,
            head_radius: head_radius,
            weapon_name: player.weapon_name,
        };
        data.push(transfer);
    }

    data
}

#[derive(serde::Serialize, serde::Deserialize, Default, Clone)]
struct BonePositions {
    bone_kind: String,
    x: f32,
    y: f32,
}

fn get_bones_pos(
    player: &Player,
    view_matrix: Matrix4<f32>,
    screen_width: usize,
    screen_height: usize,
) -> Option<(Vec<BonePositions>, f32)> {
    let bones = match player.bones_list {
        Some(ref bones) => bones,
        None => return None,
    };

    let mut head_radius: f32 = 0.0;
    let mut bones_coordinates = vec![];

    for bone in bones {
        let bone_screen_pos = world_to_screen(
            view_matrix,
            bone.matrix,
            screen_width as usize,
            screen_height as usize,
            true,
        );

        if let Some(pos) = bone_screen_pos {
            bones_coordinates.push(BonePositions {
                bone_kind: bone.kind.to_str().to_string(),
                x: pos.x,
                y: pos.y,
            });
            // if head get top and bottom of head
            if bone.kind == BoneKind::head {
                // Calculate head radius
                let head_top = Vector3::new(bone.matrix.x, bone.matrix.y, bone.matrix.z + 7.0);
                let head_bottom = Vector3::new(bone.matrix.x, bone.matrix.y, bone.matrix.z - 5.0);
                let head_top_screen_pos = world_to_screen(
                    view_matrix,
                    head_top,
                    screen_width as usize,
                    screen_height as usize,
                    true,
                );
                let head_bottom_screen_pos = world_to_screen(
                    view_matrix,
                    head_bottom,
                    screen_width as usize,
                    screen_height as usize,
                    true,
                );
                if let Some(head_top_screen_pos) = head_top_screen_pos {
                    if let Some(head_bottom_screen_pos) = head_bottom_screen_pos {
                        head_radius =
                            ((head_top_screen_pos.y - head_bottom_screen_pos.y) / 2.0).abs();
                    }
                }
            }
        }
    }

    Some((bones_coordinates, head_radius))
}

fn world_to_screen(
    view_matrix: Matrix4<f32>,
    pos: Vector3<f32>,
    screen_width: usize,
    screen_height: usize,
    always_show: bool,
) -> Option<Vector2<f32>> {
    let screen_w = (view_matrix[(3, 0)] * pos.x)
        + (view_matrix[(3, 1)] * pos.y)
        + (view_matrix[(3, 2)] * pos.z)
        + view_matrix[(3, 3)];

    if screen_w < 0.01 && !always_show {
        return None;
    }

    let screen_x = (view_matrix[(0, 0)] * pos.x)
        + (view_matrix[(0, 1)] * pos.y)
        + (view_matrix[(0, 2)] * pos.z)
        + view_matrix[(0, 3)];
    let screen_y = (view_matrix[(1, 0)] * pos.x)
        + (view_matrix[(1, 1)] * pos.y)
        + (view_matrix[(1, 2)] * pos.z)
        + view_matrix[(1, 3)];

    let cam_x = screen_width as f32 / 2.0;
    let cam_y = screen_height as f32 / 2.0;

    let x = cam_x + (cam_x * screen_x / screen_w);
    let y = cam_y - (cam_y * screen_y / screen_w);
    Some(Vector2::new(x, y))
}
