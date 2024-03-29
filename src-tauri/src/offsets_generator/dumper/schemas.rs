use super::{generate_files, Entries, Entry};

use crate::offsets_generator::builder::FileBuilderEnum;
use crate::offsets_generator::sdk::SchemaSystem;
use crate::offsets_generator::util::Process;

use anyhow::Result;

/// Dumps all schema system classes and writes the results to a file.
///
/// # Arguments
///
/// * `process` - A reference to the `Process` struct.
/// * `builders` - A mutable reference to a vector of `FileBuilderEnum`.
/// * `file_path` - A string slice representing the path to the file to write the results to.
/// * `indent` - The number of spaces to use for indentation in the output file.
///
/// # Returns
///
/// * `Result<()>` - A `Result` indicating the outcome of the operation.
pub fn dump_schemas(
    process: &Process,
    builders: &mut Vec<FileBuilderEnum>,
    indent: usize,
) -> Result<()> {
    let schema_system = SchemaSystem::new(&process)?;

    for type_scope in schema_system.type_scopes()? {
        let mut entries = Entries::new();

        for class in type_scope.classes()? {
            let parent_name = class.parent()?.map(|p| p.name().to_string());

            let container = entries.entry(class.name().replace("::", "_")).or_default();

            container.comment = parent_name;

            for field in class.fields()? {
                let name = field.name()?;
                let offset = field.offset()?;
                let type_name = field.r#type()?.name()?;

                container.data.push(Entry {
                    name,
                    value: offset as usize,
                    comment: Some(type_name),
                    indent: Some(indent),
                });
            }
        }

        generate_files(builders, &entries)?;
    }

    Ok(())
}
