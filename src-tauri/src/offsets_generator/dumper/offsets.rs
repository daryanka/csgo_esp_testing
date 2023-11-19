use super::{generate_files, Entries, Entry};

use crate::offsets_generator::builder::FileBuilderEnum;
use crate::offsets_generator::config::Operation::*;
use crate::offsets_generator::config::{Config, RAW_CONFIG};
use crate::offsets_generator::util::Process;

use anyhow::Result;

/// Dumps all offsets specified in the `config.json` file and writes the results to a file.
///
/// # Arguments
///
/// * `process` - A reference to the `Process` struct.
/// * `builders` - A mutable reference to a vector of `FileBuilderEnum`.
/// * `indent` - The number of spaces to use for indentation in the output file.
///
/// # Returns
///
/// * `Result<()>` - A `Result` indicating the outcome of the operation.
pub fn dump_offsets(
    process: &Process,
    builders: &mut Vec<FileBuilderEnum>,
    indent: usize,
) -> Result<()> {
    let config: Config = serde_json::from_str(RAW_CONFIG)?;

    let mut entries = Entries::new();

    for signature in config.signatures {
        let module = process
            .get_module_by_name(&signature.module)
            .expect(&format!("Failed to find module {}.", signature.module));

        let mut address = match process.find_pattern(&signature.module, &signature.pattern) {
            Some(a) => a,
            None => {
                continue;
            }
        };

        for operation in signature.operations {
            match operation {
                Add { value } => address += value,
                Deref { times, size } => {
                    let times = times.unwrap_or(1);
                    let size = size.unwrap_or(8);

                    for _ in 0..times {
                        process.read_memory_raw(
                            address,
                            &mut address.0 as *mut _ as *mut _,
                            size,
                        )?;
                    }
                }
                Jmp { offset, length } => {
                    address = process.resolve_jmp(address, offset, length)?.into();
                }
                Rip { offset, length } => {
                    address = process.resolve_rip(address, offset, length)?.into()
                }
                Slice { start, end } => {
                    let mut result: usize = 0;

                    process.read_memory_raw(
                        address.add(start),
                        &mut result as *mut _ as *mut _,
                        end - start,
                    )?;

                    address = result.into();
                }
                Sub { value } => address -= value,
            }
        }

        let (name, value) = if address < module.base() {
            (signature.name, address.0)
        } else {
            (signature.name, address.sub(module.base().0).0)
        };

        let container = entries
            .entry(signature.module.replace(".", "_"))
            .or_default();

        container.comment = Some(signature.module);

        container.data.push(Entry {
            name,
            value,
            comment: None,
            indent: Some(indent),
        });
    }

    generate_files(builders, &entries)?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    use core::arch::x86_64::_bittest;

    use std::ffi::{c_char, c_void};
    use std::mem::offset_of;
    use std::thread::sleep;
    use std::time::Duration;

    fn setup() -> Result<Process> {
        let mut process = Process::new("cs2.exe")?;

        process.initialize()?;

        Ok(process)
    }

    #[test]
    fn build_number() -> Result<()> {
        let process = setup()?;

        let engine_base = process
            .get_module_by_name("engine2.dll")
            .expect("Failed to find engine2.dll")
            .base();

        let build_number = process.read_memory::<u32>(engine_base + 0x48A514)?; // dwBuildNumber

        println!("Build number: {}", build_number);

        Ok(())
    }

    #[test]
    fn force_buttons() -> Result<()> {
        let process = setup()?;

        let client_base = process
            .get_module_by_name("client.dll")
            .expect("Failed to find client.dll")
            .base();

        let force_attack = process.read_memory::<u32>(client_base + 0x16B5400)?; // dwForceAttack
        let force_attack_2 = process.read_memory::<u32>(client_base + 0x16B5490)?; // dwForceAttack2
        let force_backward = process.read_memory::<u32>(client_base + 0x16B56D0)?; // dwForceBackward
        let force_crouch = process.read_memory::<u32>(client_base + 0x16B59A0)?; // dwForceCrouch
        let force_forward = process.read_memory::<u32>(client_base + 0x16B5640)?; // dwForceForward
        let force_jump = process.read_memory::<u32>(client_base + 0x16B5910)?; // dwForceJump
        let force_left = process.read_memory::<u32>(client_base + 0x16B5760)?; // dwForceLeft
        let force_right = process.read_memory::<u32>(client_base + 0x16B57F0)?; // dwForceRight

        let get_key_state = |value: u32| -> &str {
            match value {
                256 => "Key up",
                65537 => "Key down",
                _ => "Unknown",
            }
        };

        println!("Force attack: {}", get_key_state(force_attack));
        println!("Force attack 2: {}", get_key_state(force_attack_2));
        println!("Force backward: {}", get_key_state(force_backward));
        println!("Force crouch: {}", get_key_state(force_crouch));
        println!("Force forward: {}", get_key_state(force_forward));
        println!("Force jump: {}", get_key_state(force_jump));
        println!("Force left: {}", get_key_state(force_left));
        println!("Force right: {}", get_key_state(force_right));

        Ok(())
    }

    #[test]
    fn global_vars() -> Result<()> {
        #[derive(Debug)]
        #[repr(C)]
        struct GlobalVarsBase {
            real_time: f32,                  // 0x0000
            frame_count: i32,                // 0x0004
            frame_time: f32,                 // 0x0008
            absolute_frame_time: f32,        // 0x000C
            max_clients: i32,                // 0x0010
            pad_0: [u8; 0x14],               // 0x0014
            frame_time_2: f32,               // 0x0028
            current_time: f32,               // 0x002C
            current_time_2: f32,             // 0x0030
            pad_1: [u8; 0xC],                // 0x0034
            tick_count: f32,                 // 0x0040
            pad_2: [u8; 0x4],                // 0x0044
            network_channel: *const c_void,  // 0x0048
            pad_3: [u8; 0x130],              // 0x0050
            current_map: *const c_char,      // 0x0180
            current_map_name: *const c_char, // 0x0188
        }

        impl GlobalVarsBase {
            fn current_map(&self, process: &Process) -> Result<String> {
                let name_ptr = process.read_memory::<usize>(
                    (self as *const _ as usize + offset_of!(Self, current_map)).into(),
                )?;

                process.read_string(name_ptr.into())
            }

            fn current_map_name(&self, process: &Process) -> Result<String> {
                let name_ptr = process.read_memory::<usize>(
                    (self as *const _ as usize + offset_of!(Self, current_map_name)).into(),
                )?;

                process.read_string(name_ptr.into())
            }
        }

        let process = setup()?;

        let client_base = process
            .get_module_by_name("client.dll")
            .expect("Failed to find client.dll")
            .base();

        let global_vars = process.read_memory::<*const GlobalVarsBase>(client_base + 0x16B14E0)?; // dwGlobalVars

        let current_map_name = unsafe {
            (*global_vars)
                .current_map_name(&process)
                .unwrap_or_default()
        };

        println!("Current map name: {}", current_map_name);

        Ok(())
    }

    #[test]
    fn is_key_down() -> Result<()> {
        let process = setup()?;

        let input_system_base = process
            .get_module_by_name("inputsystem.dll")
            .expect("Failed to find inputsystem.dll")
            .base();

        let input_system = input_system_base + 0x35760; // dwInputSystem

        let is_key_down = |key_code: i32| -> bool {
            let key_map_element = process
                .read_memory::<i32>((input_system + 0x4 * (key_code as usize / 32) + 0x12A0).into())
                .unwrap_or(0);

            unsafe { _bittest(&key_map_element, key_code & 0x1F) != 0 }
        };

        sleep(Duration::from_secs(1));

        // See https://www.unknowncheats.me/forum/3855779-post889.html for button codes.
        println!("Insert down: {}", is_key_down(73));

        Ok(())
    }

    #[test]
    fn local_player_controller() -> Result<()> {
        let process = setup()?;

        let client_base = process
            .get_module_by_name("client.dll")
            .expect("Failed to find client.dll")
            .base();

        let local_player_controller = process.read_memory::<usize>(client_base + 0x1800008)?; // dwLocalPlayerController

        let player_name = process.read_string((local_player_controller + 0x640).into())?; // m_iszPlayerName

        println!("Local player name: {}", player_name);

        Ok(())
    }

    #[test]
    fn local_player_pawn() -> Result<()> {
        let process = setup()?;

        let client_base = process
            .get_module_by_name("client.dll")
            .expect("Failed to find client.dll")
            .base();

        let local_player_pawn = process.read_memory::<usize>(client_base + 0x16BC4A8)?; // dwLocalPlayerPawn

        let game_scene_node = process.read_memory::<usize>((local_player_pawn + 0x310).into())?; // m_pGameSceneNode

        #[derive(Debug)]
        #[repr(C)]
        struct Vector3D {
            x: f32,
            y: f32,
            z: f32,
        }

        let origin = process.read_memory::<Vector3D>((game_scene_node + 0xC8).into())?; // m_vecAbsOrigin

        println!("Local player origin: {:?}", origin);

        Ok(())
    }

    #[test]
    fn window_size() -> Result<()> {
        let process = setup()?;

        let engine_base = process
            .get_module_by_name("engine2.dll")
            .expect("Failed to find engine2.dll")
            .base();

        let window_width = process.read_memory::<u32>(engine_base + 0x540CE0)?; // dwWindowWidth
        let window_height = process.read_memory::<u32>(engine_base + 0x540CE4)?; // dwWindowHeight

        println!("Window size: {}x{}", window_width, window_height);

        Ok(())
    }
}
