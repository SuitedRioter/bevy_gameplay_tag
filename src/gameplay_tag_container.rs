use crate::gameplay_tag::GameplayTag;
use crate::gameplay_tags_manager::GameplayTagsManager;
use bevy::prelude::Component;
use bevy::prelude::Res;

#[derive(Component, Debug)]
pub struct GameplayTagContainer {
    pub gameplay_tags: Vec<GameplayTag>,
    pub parent_tags: Vec<GameplayTag>,
}

impl Default for GameplayTagContainer {
    fn default() -> Self {
        GameplayTagContainer::new()
    }
}

impl GameplayTagContainer {
    pub fn new() -> Self {
        GameplayTagContainer {
            gameplay_tags: Vec::new(),
            parent_tags: Vec::new(),
        }
    }

    pub fn is_empty(&self) -> bool {
        self.gameplay_tags.is_empty()
    }

    pub fn reset(&mut self) {
        self.gameplay_tags.clear();
        self.parent_tags.clear();
    }

    pub fn find_tag_index(&self, tag: &GameplayTag) -> Option<usize> {
        self.gameplay_tags.binary_search(tag).ok()
    }

    pub fn has_tag(&self, tag: &GameplayTag) -> bool {
        self.gameplay_tags.binary_search(tag).is_ok() || self.parent_tags.binary_search(tag).is_ok()
    }

    pub fn has_tag_exact(&self, tag: &GameplayTag) -> bool {
        self.gameplay_tags.binary_search(tag).is_ok()
    }

    pub fn has_any(&self, container_to_check: &GameplayTagContainer) -> bool {
        if container_to_check.is_empty() {
            false
        } else {
            container_to_check
                .gameplay_tags
                .iter()
                .any(|tag| self.has_tag(tag))
        }
    }

    pub fn has_any_exact(&self, container_to_check: &GameplayTagContainer) -> bool {
        if container_to_check.is_empty() {
            false
        } else {
            container_to_check
                .gameplay_tags
                .iter()
                .any(|tag| self.has_tag_exact(tag))
        }
    }

    pub fn has_all(&self, container_to_check: &GameplayTagContainer) -> bool {
        if container_to_check.is_empty() {
            true
        } else {
            container_to_check
                .gameplay_tags
                .iter()
                .all(|tag| self.has_tag(tag))
        }
    }

    pub fn has_all_exact(&self, container_to_check: &GameplayTagContainer) -> bool {
        if container_to_check.is_empty() {
            true
        } else {
            container_to_check
                .gameplay_tags
                .iter()
                .all(|tag| self.has_tag_exact(tag))
        }
    }

    pub fn add_tag(&mut self, tag: GameplayTag, tags_manager: &Res<GameplayTagsManager>) {
        if tag.is_valid() {
            match self.gameplay_tags.binary_search(&tag) {
                Ok(_) => {}
                Err(index) => {
                    self.gameplay_tags.insert(index, tag.clone());
                    self.add_parent_tag(tag, tags_manager);
                }
            }
        }
    }

    pub fn add_tag_fast(&mut self, tag: GameplayTag, tags_manager: &Res<GameplayTagsManager>) {
        match self.gameplay_tags.binary_search(&tag) {
            Ok(_) => {}
            Err(index) => {
                self.gameplay_tags.insert(index, tag.clone());
                self.add_parent_tag(tag, tags_manager);
            }
        }
    }

    pub fn add_parent_tag(&mut self, tag: GameplayTag, tags_manager: &Res<GameplayTagsManager>) {
        let complete_container = tags_manager.get_single_tag_container(&tag);
        if let Some(exist_container) = complete_container {
            for tag in exist_container.parent_tags.iter() {
                match self.parent_tags.binary_search(tag) {
                    Ok(_) => {}
                    Err(index) => self.parent_tags.insert(index, tag.clone()),
                }
            }
        }
    }

    pub fn fill_parent_tags(&mut self, tags_manager: &Res<GameplayTagsManager>) {
        self.parent_tags.clear();
        for tag in self.gameplay_tags.iter() {
            let complete_container = tags_manager.get_single_tag_container(tag);
            if let Some(exist_container) = complete_container {
                for parent_tag in exist_container.parent_tags.iter() {
                    match self.parent_tags.binary_search(parent_tag) {
                        Ok(_) => {}
                        Err(index) => self.parent_tags.insert(index, parent_tag.clone()),
                    }
                }
            }
        }
    }

    pub fn remove_tag(
        &mut self,
        tag: &GameplayTag,
        defer_parent_tags: bool,
        tags_manager: &Res<GameplayTagsManager>,
    ) -> bool {
        let index = self.find_tag_index(tag);
        match index {
            Some(index) => {
                self.gameplay_tags.remove(index);
                if !defer_parent_tags {
                    self.fill_parent_tags(tags_manager);
                }
                true
            }
            None => false,
        }
    }

    pub fn remove_tags(
        &mut self,
        tags_to_remove: GameplayTagContainer,
        tags_manager: &Res<GameplayTagsManager>,
    ) {
        let mut num_changed = 0;
        for tag in tags_to_remove.gameplay_tags.iter() {
            let index = self.find_tag_index(tag);
            match index {
                Some(index) => {
                    self.gameplay_tags.remove(index);
                    num_changed += 1;
                }
                None => continue,
            }
        }
        if num_changed > 0 {
            self.fill_parent_tags(tags_manager);
        }
    }

    pub fn append_matches_tags(
        &mut self,
        other_a: &GameplayTagContainer,
        other_b: &GameplayTagContainer,
        tags_manager: &Res<GameplayTagsManager>,
    ) {
        for other_a_tag in other_a.gameplay_tags.iter() {
            if other_a_tag.matches_any(other_b, tags_manager) {
                self.add_tag(other_a_tag.clone(), tags_manager);
            }
        }
    }

    pub fn append_tags(
        &mut self,
        other: &GameplayTagContainer,
        tags_manager: &Res<GameplayTagsManager>,
    ) {
        for tag in other.gameplay_tags.iter() {
            self.add_tag(tag.clone(), tags_manager);
        }
    }

    pub fn filter(
        &self,
        other: &GameplayTagContainer,
        tags_manager: &Res<GameplayTagsManager>,
    ) -> GameplayTagContainer {
        let mut filtered_tags = GameplayTagContainer::new();
        for tag in self.gameplay_tags.iter() {
            if tag.matches_any(other, tags_manager) {
                filtered_tags.add_tag(tag.clone(), tags_manager);
            }
        }
        filtered_tags
    }

    pub fn filter_exact(
        &self,
        other: &GameplayTagContainer,
        tags_manager: &Res<GameplayTagsManager>,
    ) -> GameplayTagContainer {
        let mut filtered_tags = GameplayTagContainer::new();
        for tag in self.gameplay_tags.iter() {
            if tag.matches_any_exact(other) {
                filtered_tags.add_tag(tag.clone(), tags_manager);
            }
        }
        filtered_tags
    }

    /// 返回容器的所有标签，包括显示标签和隐式标签。
    /// 需要注意，返回的容器只有 gameplay_tags 属性有值，就是标签全部放在 gameplay_tags 属性里面的。
    pub fn get_gameplay_tag_parents(&self) -> GameplayTagContainer {
        let mut result_container = GameplayTagContainer::new();
        result_container.gameplay_tags = self.gameplay_tags.clone();
        for tag in self.parent_tags.iter() {
            match result_container.gameplay_tags.binary_search(tag) {
                Ok(_) => {}
                Err(index) => result_container.gameplay_tags.insert(index, tag.clone()),
            }
        }
        result_container
    }
}
