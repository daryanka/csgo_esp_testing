pub use file_builder::FileBuilder;
pub use json_file_builder::JsonFileBuilder;

pub use std::io::{Result, Write};

pub mod file_builder;
pub mod json_file_builder;

/// `FileBuilder` is an enum that defines different kinds of file builders.
/// Each variant corresponds to a builder for a particular type of file.
#[derive(Clone, Debug, PartialEq)]
pub enum FileBuilderEnum {
    /// Represents a builder for JSON files.
    JsonFileBuilder(JsonFileBuilder),
}

impl FileBuilder for FileBuilderEnum {
    fn extension(&mut self) -> &str {
        self.as_mut().extension()
    }

    fn write_top_level(&mut self, output: &mut dyn Write) -> Result<()> {
        self.as_mut().write_top_level(output)
    }

    fn write_namespace(&mut self, name: &str, comment: Option<&str>) -> Result<()> {
        self.as_mut().write_namespace(name, comment)
    }

    fn write_variable(
        &mut self,
        name: &str,
        value: usize,
        comment: Option<&str>,
        indentation: Option<usize>,
    ) -> Result<()> {
        self.as_mut()
            .write_variable(name, value, comment, indentation)
    }

    fn print(&self) -> () {
        self.as_mut().print()
    }
}

impl FileBuilderEnum {
    fn as_mut(&mut self) -> &mut dyn FileBuilder {
        match self {
            FileBuilderEnum::JsonFileBuilder(builder) => builder,
        }
    }
}
