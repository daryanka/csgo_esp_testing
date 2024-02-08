#![allow(dead_code)]

use anyhow::Result;

use builder::*;

use dumper::{dump_interfaces, dump_offsets, dump_schemas};

use util::Process;

use crate::csgo::DynamicOffsets;

mod builder;
mod config;
mod dumper;
mod sdk;
mod util;

/// Command line arguments for the program.
pub struct Args {
    pub interfaces: bool,
    pub offsets: bool,
    pub schemas: bool,
}

pub fn get_offsets(
    Args {
        interfaces,
        offsets,
        schemas,
    }: Args,
) -> Result<DynamicOffsets> {
    let mut builders: Vec<FileBuilderEnum> =
        vec![FileBuilderEnum::JsonFileBuilder(JsonFileBuilder::default())];
    let mut process = Process::new("cs2.exe")?;
    let indent = 4;

    process.initialize()?;

    let all = !(interfaces || offsets || schemas);

    if schemas || all {
        dump_schemas(&mut process, &mut builders, indent)?;
    }

    if interfaces || all {
        dump_interfaces(&mut process, &mut builders, indent)?;
    }

    if offsets || all {
        dump_offsets(&mut process, &mut builders, indent)?;
    }

    let res = builders[0].generate();
    if let Some(offset_data) = res {
        return Ok(offset_data.get_dynamic_offsets());
    } else {
        Err(anyhow::anyhow!("Failed to generate offsets"))
    }
}

/// Parses the given file extension and returns the corresponding `FileBuilderEnum`.
///
/// # Arguments
///
/// * `extension` - A string slice that represents the file extension.
///
/// # Returns
///
/// * `Ok(FileBuilderEnum)` - If the extension is valid, returns the corresponding `FileBuilderEnum`.
/// * `Err(&'static str)` - If the extension is invalid, returns an error message.
fn parse_extension(extension: &str) -> Result<FileBuilderEnum, &'static str> {
    match extension {
        ".json" => Ok(FileBuilderEnum::JsonFileBuilder(JsonFileBuilder::default())),
        _ => Err("Invalid extension"),
    }
}
