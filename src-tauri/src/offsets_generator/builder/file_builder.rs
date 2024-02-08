use std::{
    collections::HashMap,
    io::{Result, Write},
};

use serde::{Deserialize, Serialize};

use crate::csgo::DynamicOffsets;

#[derive(Serialize, Deserialize)]
pub struct OffsetData(HashMap<String, DataLayer>);

#[derive(Serialize, Deserialize)]
pub struct DataLayer {
    pub data: HashMap<String, ValueLayer>,
}

#[derive(Serialize, Deserialize)]
pub struct ValueLayer {
    pub value: usize,
}

impl OffsetData {
    pub fn get_dynamic_offsets(&self) -> DynamicOffsets {
        let mut data = DynamicOffsets::default();

        if let Some(entity_list) = self.0.get("client_dll") {
            //dwEntityList
            if let Some(entity_list_data) = entity_list.data.get("dwEntityList") {
                data.dwEntityList = entity_list_data.value;
            }
            //dwLocalPlayerController
            if let Some(local_player_controller_data) =
                entity_list.data.get("dwLocalPlayerController")
            {
                data.dwLocalPlayerController = local_player_controller_data.value;
            }

            //dwViewMatrix
            if let Some(view_matrix_data) = entity_list.data.get("dwViewMatrix") {
                data.dwViewMatrix = view_matrix_data.value;
            }
        }

        if let Some(entity_list) = self.0.get("C_BaseEntity") {
            //m_iHealth
            if let Some(entity_list_data) = entity_list.data.get("m_iHealth") {
                data.m_iHealth = entity_list_data.value;
            }
            //m_iTeamNum
            if let Some(entity_list_data) = entity_list.data.get("m_iTeamNum") {
                data.m_iTeamNum = entity_list_data.value;
            }
        }

        if let Some(entity_list) = self.0.get("CCSPlayerController") {
            //m_hPlayerPawn
            if let Some(entity_list_data) = entity_list.data.get("m_hPlayerPawn") {
                data.m_hPlayerPawn = entity_list_data.value;
            }
        }

        if let Some(entity_list) = self.0.get("CGameSceneNode") {
            //m_vecAbsOrigin
            if let Some(entity_list_data) = entity_list.data.get("m_vecAbsOrigin") {
                data.m_vecAbsOrigin = entity_list_data.value;
            }
        }

        if let Some(entity_list) = self.0.get("C_BasePlayerPawn") {
            //m_vOldOrigin
            if let Some(entity_list_data) = entity_list.data.get("m_vOldOrigin") {
                data.m_vOldOrigin = entity_list_data.value;
            }
        }

        if let Some(entity_list) = self.0.get("C_CSPlayerPawnBase") {
            //m_pClippingWeapon
            if let Some(entity_list_data) = entity_list.data.get("m_pClippingWeapon") {
                data.m_pClippingWeapon = entity_list_data.value;
            }
        }

        data
    }
}

/// A trait that defines the file builder operations.
pub trait FileBuilder {
    /// Returns the extension of the file.
    ///
    /// # Arguments
    ///
    /// * `&mut self` - A mutable reference to the `FileBuilder` struct.
    ///
    /// # Returns
    ///
    /// * `&str` - A string slice containing the extension of the file.
    fn extension(&mut self) -> &str;

    /// Writes to the top level of the file. The output destination is `output`.
    ///
    /// # Arguments
    ///
    /// * `&mut self` - A mutable reference to the `FileBuilder` struct.
    /// * `output` - An object implementing Write trait where the top level will be written.
    ///
    /// # Returns
    ///
    /// * `Result<()>` - A generic Result type indicating the operations outcome.
    fn write_top_level(&mut self, output: &mut dyn Write) -> Result<()>;

    /// Writes a namespace to the output.
    ///
    /// # Arguments
    ///
    /// * `&mut self` - A mutable reference to the `FileBuilder` struct.
    /// * `output` - An object implementing Write trait where the namespace will be written.
    /// * `name` - The name of the namespace.
    /// * `comment` - An optional comment. If present, this comment will be included in the output.
    ///
    /// # Returns
    ///
    /// * `Result<()>` - A generic Result type indicating the operations outcome.
    fn write_namespace(&mut self, name: &str, comment: Option<&str>) -> Result<()>;

    /// Writes a variable to the output.
    ///
    /// # Arguments
    ///
    /// * `&mut self` - A mutable reference to the `FileBuilder` struct.
    /// * `output` - An object implementing Write trait where the variable will be written.
    /// * `name` - The name of the variable.
    /// * `value` - The value of the variable.
    /// * `comment` - An optional comment. If present, this comment will be included in the output.
    /// * `indentation` - An optional indentation value. If present, the variable will be written with the specified indentation.
    ///
    /// # Returns
    ///
    /// * `Result<()>` - A generic Result type indicating the operations outcome.
    fn write_variable(
        &mut self,
        name: &str,
        value: usize,
        comment: Option<&str>,
        indentation: Option<usize>,
    ) -> Result<()>;

    fn generate(&mut self) -> Option<OffsetData>;
}
