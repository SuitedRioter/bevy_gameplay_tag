use bevy::{ecs::world::World, prelude::Res};
use std::hash::{Hash, Hasher};

use string_cache::DefaultAtom as FName;

use crate::{
    gameplay_tag_container::GameplayTagContainer, gameplay_tags_manager::GameplayTagsManager,
};

#[derive(Debug, Eq, Clone, Ord, PartialOrd)]
pub struct GameplayTag {
    //标签完整名字
    tag_name: FName,
}

impl PartialEq for GameplayTag {
    fn eq(&self, other: &Self) -> bool {
        self.tag_name == other.tag_name
    }
}

impl Hash for GameplayTag {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.tag_name.hash(state);
    }
}

impl GameplayTag {
    pub fn new(full_name: FName) -> GameplayTag {
        GameplayTag {
            tag_name: full_name,
        }
    }

    pub fn get_tag_name(&self) -> &str {
        &self.tag_name
    }

    pub fn is_valid(&self) -> bool {
        !self.tag_name.is_empty()
    }

    pub fn matches_tag(
        &self,
        tag_to_check: &GameplayTag,
        tags_manager: &Res<GameplayTagsManager>,
        world: &World,
    ) -> bool {
        let complete_container = tags_manager.get_single_tag_container(self, world);
        if let Some(exist_container) = complete_container {
            exist_container.has_tag(tag_to_check)
        } else {
            false
        }
    }

    pub fn matches_tag_exact(&self, tag_to_check: &GameplayTag) -> bool {
        if !tag_to_check.is_valid() {
            false
        } else {
            self.tag_name == tag_to_check.tag_name
        }
    }

    pub fn matches_any(
        &self,
        container_to_check: &GameplayTagContainer,
        tags_manager: &Res<GameplayTagsManager>,
        world: &World,
    ) -> bool {
        let complete_container = tags_manager.get_single_tag_container(self, world);
        if let Some(exist_container) = complete_container {
            exist_container.has_any(container_to_check)
        } else {
            false
        }
    }

    pub fn matches_any_exact(&self, container_to_check: &GameplayTagContainer) -> bool {
        if container_to_check.is_empty() {
            false
        } else {
            container_to_check.gameplay_tags.binary_search(self).is_ok()
        }
    }
}
