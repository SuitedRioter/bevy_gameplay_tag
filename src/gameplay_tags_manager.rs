use crate::gameplay_tag::GameplayTag;
use crate::gameplay_tag_container::GameplayTagContainer;
use bevy::platform::collections::HashMap;
use bevy::prelude::{ChildOf, Children, Component, Entity, FromWorld, Name, Resource, World};
use bevy::log::error;
use serde::{Deserialize, Serialize};
use std::{fs::read_to_string, io};
use string_cache::DefaultAtom as FName;

#[derive(Debug)]
pub enum GameplayTagsLoadError {
    Io(io::Error),
    Parse(serde_json::Error),
}

impl std::fmt::Display for GameplayTagsLoadError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            GameplayTagsLoadError::Io(error) => write!(f, "failed to read tag data: {error}"),
            GameplayTagsLoadError::Parse(error) => write!(f, "failed to parse tag data: {error}"),
        }
    }
}

impl std::error::Error for GameplayTagsLoadError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            GameplayTagsLoadError::Io(error) => Some(error),
            GameplayTagsLoadError::Parse(error) => Some(error),
        }
    }
}

impl From<io::Error> for GameplayTagsLoadError {
    fn from(value: io::Error) -> Self {
        GameplayTagsLoadError::Io(value)
    }
}

impl From<serde_json::Error> for GameplayTagsLoadError {
    fn from(value: serde_json::Error) -> Self {
        GameplayTagsLoadError::Parse(value)
    }
}

#[derive(Resource, Debug)]
pub struct GameplayTagsManager {
    pub root: Entity,
    pub tag_map: HashMap<GameplayTag, GameplayTagContainer>,
}

impl FromWorld for GameplayTagsManager {
    fn from_world(world: &mut World) -> Self {
        let tag_settings = world
            .remove_resource::<GameplayTagsSettings>()
            .unwrap_or_default();

        let tag_data_table = if let Some(data_path) = &tag_settings.data_path {
            match GameplayTagsSettings::load_tag_table_from_path(data_path) {
                Ok(data) => data,
                Err(e) => {
                    error!("Failed to load tag data from {}: {}", data_path, e);
                    Vec::new()
                }
            }
        } else {
            match GameplayTagsSettings::parse_tag_table(&tag_settings.json_data) {
                Ok(data) => data,
                Err(e) => {
                    error!("Failed to parse tag data from json_data: {}", e);
                    Vec::new()
                }
            }
        };

        let root = world
            .spawn((
                GameplayTagNode::new(FName::from("Root"), false),
                Name::new("Root"),
            ))
            .id();

        let mut gameplay_tags_manager = GameplayTagsManager {
            root,
            tag_map: HashMap::new(),
        };

        for data_row in tag_data_table {
            gameplay_tags_manager.add_tag_node(data_row.tag_name, world);
        }

        gameplay_tags_manager
    }
}

impl GameplayTagsManager {
    pub fn get_single_tag_container(&self, tag: &GameplayTag) -> Option<&GameplayTagContainer> {
        self.tag_map.get(tag)
    }

    pub fn request_gameplay_tag_parents(&self, tag: &GameplayTag) -> GameplayTagContainer {
        let parent_tags = self.get_single_tag_container(tag);
        if let Some(exist_tags) = parent_tags {
            exist_tags.get_gameplay_tag_parents()
        } else {
            GameplayTagContainer::new()
        }
    }

    pub fn parent_tags(&self, tag: &GameplayTag) -> GameplayTagContainer {
        self.request_gameplay_tag_parents(tag)
    }

    pub fn parents_of(&self, tag: &GameplayTag) -> GameplayTagContainer {
        self.request_gameplay_tag_parents(tag)
    }

    fn add_tag_node(&mut self, tag_name: String, world: &mut World) {
        let mut current_node_entity = self.root;
        let parts: Vec<&str> = tag_name.split(".").collect();
        let mut full_tag_string = String::new();

        for (index, part) in parts.iter().enumerate() {
            let is_explicit = index == parts.len() - 1;
            let short_tag_name = part.to_string();

            // 构建完整标签名
            if index == 0 {
                full_tag_string = short_tag_name.clone();
            } else {
                full_tag_string = format!("{}.{}", full_tag_string, short_tag_name);
            }

            // 查找是否已存在
            let child_entity = self.find_child_by_name(world, current_node_entity, &short_tag_name);
            if let Some(existing_child) = child_entity {
                current_node_entity = existing_child;
                if is_explicit
                    && let Some(mut node) = world.get_mut::<GameplayTagNode>(current_node_entity)
                {
                    node.is_explicit_tag = true;
                }
            } else {
                let complete_container = self.build_complete_tag_container(&full_tag_string);
                let new_node_entity = world
                    .spawn((
                        GameplayTagNode {
                            tag_name: FName::from(short_tag_name),
                            is_explicit_tag: is_explicit,
                        },
                        ChildOf(current_node_entity),
                        Name::new(full_tag_string.clone()),
                    ))
                    .id();
                let gameplay_tag_to_node = GameplayTag::new(full_tag_string.clone().as_str());
                self.tag_map
                    .insert(gameplay_tag_to_node, complete_container);

                current_node_entity = new_node_entity;
            }
        }
    }

    fn build_complete_tag_container(&self, full_tag_name: &str) -> GameplayTagContainer {
        let mut container = GameplayTagContainer::new();
        let self_tag = GameplayTag::new(full_tag_name);
        container.gameplay_tags.push(self_tag);
        let parts: Vec<&str> = full_tag_name.split('.').collect();
        let mut current_path = String::new();
        for (index, part) in parts.iter().enumerate() {
            let short_tag_name = part.to_string();
            // 设置当前节点的标签全名 (current_node)
            if index == 0 {
                current_path = short_tag_name.clone();
            } else {
                current_path = format!("{}.{}", current_path, short_tag_name);
            }

            // 跳过最后一个（自己已经添加过了）
            if index < parts.len() - 1 {
                let parent_tag = GameplayTag::new(current_path.clone().as_str());
                container.parent_tags.push(parent_tag);
            }
        }

        container
    }

    fn find_child_by_name(&self, world: &World, parent: Entity, name: &str) -> Option<Entity> {
        if let Some(children) = world.get::<Children>(parent) {
            for child in children.iter() {
                if let Some(child_node) = world.get::<GameplayTagNode>(*child)
                    && child_node.tag_name.as_ref() == name
                {
                    return Some(*child);
                }
            }
        }
        None
    }
}

#[derive(Debug, Component)]
pub struct GameplayTagNode {
    //不是标签完整名字，当前节点的名字
    tag_name: FName,
    is_explicit_tag: bool,
}

impl GameplayTagNode {
    fn new(short_name: FName, is_explicit_tag: bool) -> Self {
        GameplayTagNode {
            tag_name: short_name,
            is_explicit_tag,
        }
    }
}

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq, Eq)]
pub struct GameplayTagTableRow {
    pub tag_name: String,
    pub description: String,
}

#[derive(Resource, Debug)]
pub struct GameplayTagsSettings {
    pub json_data: String,
    pub data_path: Option<String>,
}

impl Default for GameplayTagsSettings {
    fn default() -> Self {
        GameplayTagsSettings {
            json_data: r#"
            [
                { "tag_name": "A.B.C", "description": "Description of A.B.C" },
                { "tag_name": "A.B.D", "description": "Description of A.B.D" },
                { "tag_name": "A.C", "description": "Description of A.C" },
                { "tag_name": "D", "description": "Description of D" },
                { "tag_name": "D.C", "description": "Description of D" },
                { "tag_name": "D.C.B", "description": "Description of D" },
                { "tag_name": "A.C.B", "description": "Description of D" },
                { "tag_name": "Status.Damaged",  "description": "Damaged" },
                { "tag_name": "Buff.Strength",  "description": "Buff.Strength" }
            ]
            "#
            .to_string(),
            data_path: None,
        }
    }
}

impl GameplayTagsSettings {
    pub fn new() -> Self {
        GameplayTagsSettings::default()
    }

    pub fn parse_tag_table(json_data: &str) -> Result<Vec<GameplayTagTableRow>, GameplayTagsLoadError> {
        Ok(serde_json::from_str(json_data)?)
    }

    pub fn load_tag_table_from_path(
        data_path: impl AsRef<std::path::Path>,
    ) -> Result<Vec<GameplayTagTableRow>, GameplayTagsLoadError> {
        let json_content = read_to_string(data_path)?;
        Self::parse_tag_table(&json_content)
    }

    pub fn with_data_path(data_path: impl Into<String>) -> Self {
        GameplayTagsSettings {
            data_path: Some(data_path.into()),
            json_data: String::new(),
        }
    }

    pub fn from_path(data_path: impl Into<String>) -> Self {
        Self::with_data_path(data_path)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_tag_table_returns_rows() {
        let rows = GameplayTagsSettings::parse_tag_table(
            r#"[{"tag_name":"Ability.Skill.Fire","description":"Fire skill"}]"#,
        )
        .unwrap();

        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].tag_name, "Ability.Skill.Fire");
    }

    #[test]
    fn parse_tag_table_reports_errors() {
        let error = GameplayTagsSettings::parse_tag_table("not json").unwrap_err();
        assert!(matches!(error, GameplayTagsLoadError::Parse(_)));
    }

    #[test]
    fn path_helpers_store_string_inputs() {
        let settings = GameplayTagsSettings::with_data_path("tags.json");
        assert_eq!(settings.data_path.as_deref(), Some("tags.json"));

        let from_path = GameplayTagsSettings::from_path(String::from("tags.json"));
        assert_eq!(from_path.data_path.as_deref(), Some("tags.json"));
    }
}
