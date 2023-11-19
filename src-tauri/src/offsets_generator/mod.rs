#![allow(dead_code)]

use anyhow::Result;

use builder::*;

use dumper::{dump_interfaces, dump_offsets, dump_schemas};

use std::fs;
use std::time::Instant;

use util::Process;

mod builder;
mod config;
mod dumper;
mod sdk;
mod util;

/// Command line arguments for the program.
struct Args {
    interfaces: bool,
    offsets: bool,
    schemas: bool,
    builders: Vec<FileBuilderEnum>,
    indent: usize,
    output: String,
    verbose: bool,
}

pub fn get_offsets(
    Args {
        interfaces,
        offsets,
        schemas,
        mut builders,
        indent,
        output,
        verbose,
    }: Args,
) -> Result<()> {
    let now = Instant::now();

    fs::create_dir_all(&output)?;

    let mut process = Process::new("cs2.exe")?;

    process.initialize()?;

    let all = !(interfaces || offsets || schemas);

    if schemas || all {
        dump_schemas(&mut process, &mut builders, &output, indent)?;
    }

    if interfaces || all {
        dump_interfaces(&mut process, &mut builders, &output, indent)?;
    }

    if offsets || all {
        dump_offsets(&mut process, &mut builders, &output, indent)?;
    }

    Ok(())
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
        ".cs" => Ok(FileBuilderEnum::CSharpFileBuilder(CSharpFileBuilder)),
        ".hpp" => Ok(FileBuilderEnum::CppFileBuilder(CppFileBuilder)),
        ".json" => Ok(FileBuilderEnum::JsonFileBuilder(JsonFileBuilder::default())),
        ".py" => Ok(FileBuilderEnum::PythonFileBuilder(PythonFileBuilder)),
        ".rs" => Ok(FileBuilderEnum::RustFileBuilder(RustFileBuilder)),
        _ => Err("Invalid extension"),
    }
}
