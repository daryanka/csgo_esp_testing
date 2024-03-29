use super::file_builder::OffsetData;
use super::FileBuilder;

use serde::Serialize;

use std::collections::BTreeMap;
use std::io::{Result, Write};

/// Represents a JSON offset value with an optional comment.
#[derive(Clone, Debug, Default, PartialEq, Serialize)]
struct JsonOffsetValue {
    value: usize,
    comment: Option<String>,
}

/// Represents a JSON module, which contains data in the form of a `BTreeMap` of string keys and
/// `JsonOffsetValue` values, as well as an optional comment.
#[derive(Clone, Debug, Default, PartialEq, Serialize)]
struct JsonModule {
    data: BTreeMap<String, JsonOffsetValue>,
    comment: Option<String>,
}

/// A structure representing a builder for JSON files.
/// The builder implements the `FileBuilder` trait.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct JsonFileBuilder {
    data: BTreeMap<String, JsonModule>,
    current_namespace: String,
}

impl FileBuilder for JsonFileBuilder {
    fn extension(&mut self) -> &str {
        "json"
    }

    fn write_top_level(&mut self, _output: &mut dyn Write) -> Result<()> {
        Ok(())
    }

    fn write_namespace(&mut self, name: &str, comment: Option<&str>) -> Result<()> {
        self.data.entry(name.to_string()).or_default().comment = comment.map(str::to_string);
        self.current_namespace = name.to_string();

        Ok(())
    }

    fn write_variable(
        &mut self,
        name: &str,
        value: usize,
        comment: Option<&str>,
        _indentation: Option<usize>,
    ) -> Result<()> {
        self.data
            .entry(self.current_namespace.clone())
            .or_default()
            .data
            .insert(
                name.to_string(),
                JsonOffsetValue {
                    value,
                    comment: comment.map(str::to_string),
                },
            );

        Ok(())
    }

    fn generate(&mut self) -> Option<OffsetData> {
        let json_str = match serde_json::to_string(&self.data) {
            Ok(json_str) => json_str,
            Err(_) => {
                return None;
            }
        };
        let data: OffsetData = match serde_json::from_str(&json_str) {
            Ok(data) => data,
            Err(_) => {
                println!("Error: Failed to parse JSON string");
                return None;
            }
        };
        Some(data)
    }
}
