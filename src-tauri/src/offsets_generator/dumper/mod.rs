use crate::offsets_generator::builder::{FileBuilder, FileBuilderEnum};

use anyhow::Result;

pub use interfaces::dump_interfaces;
pub use offsets::dump_offsets;
pub use schemas::dump_schemas;

use std::collections::BTreeMap;

pub mod interfaces;
pub mod offsets;
pub mod schemas;

/// Represents an entry in the generated file.
#[derive(Debug, PartialEq)]
pub struct Entry {
    /// The name of the entry.
    pub name: String,

    /// The value of the entry.
    pub value: usize,

    /// An optional comment associated with the entry.
    pub comment: Option<String>,

    /// An optional indentation level for the entry.
    pub indent: Option<usize>,
}

/// A container for entries, which consists of data and an optional comment.
#[derive(Default)]
pub struct EntriesContainer {
    /// The data associated with the container.
    pub data: Vec<Entry>,

    /// An optional comment associated with the container.
    pub comment: Option<String>,
}

/// A type alias for a `BTreeMap` that maps `String` keys to `EntriesContainer` values.
pub type Entries = BTreeMap<String, EntriesContainer>;

/// Generates a file using the given `builder`, `entries`, `file_path`, and `file_name`.
///
/// # Arguments
///
/// * `builder` - A mutable reference to the `FileBuilderEnum`.
/// * `entries` - A reference to the `Entries` struct.
///
/// # Returns
///
/// * `Result<()>` - A `Result` indicating the outcome of the operation.
pub fn generate_file(builder: &mut FileBuilderEnum, entries: &Entries) -> Result<()> {
    if entries.is_empty() {
        return Ok(());
    }

    let len = entries.len();

    for (i, pair) in entries.iter().enumerate() {
        builder.write_namespace(pair.0, pair.1.comment.as_deref())?;

        pair.1.data.iter().try_for_each(|entry| {
            builder.write_variable(
                &entry.name,
                entry.value,
                entry.comment.as_deref(),
                entry.indent,
            )
        })?;
    }

    builder.print();

    Ok(())
}

/// Generate files using the given `builders`, `entries`, `file_path`, and `file_name`.
///
/// # Arguments
///
/// * `builders` - A mutable slice of `FileBuilderEnum` objects.
/// * `entries` - A reference to the `Entries` struct.
/// * `file_path` - A string slice representing the path to the file.
/// * `file_name` - A string slice representing the name of the file.
///
/// # Returns
///
/// * `Result<()>` - A `Result` indicating the outcome of the operation.
pub fn generate_files(builders: &mut [FileBuilderEnum], entries: &Entries) -> Result<()> {
    builders
        .iter_mut()
        .try_for_each(|builder| generate_file(builder, entries))
}
