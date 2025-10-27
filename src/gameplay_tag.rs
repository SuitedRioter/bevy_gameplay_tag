use std::fmt::Debug;
use bevy::prelude::Res;
use std::hash::{Hash, Hasher};

use string_cache::DefaultAtom as FName;

use crate::{
    gameplay_tag_container::GameplayTagContainer, gameplay_tags_manager::GameplayTagsManager,
};

#[derive(Eq, Clone, Ord, PartialOrd)]
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

impl Debug for GameplayTag {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.tag_name.as_ref())
    }
}

impl GameplayTag {
    pub fn new(full_name: &str) -> GameplayTag {
        GameplayTag {
            tag_name: FName::from(full_name),
        }
    }

    pub fn get_tag_name(&self) -> &str {
        &self.tag_name
    }

    pub fn is_valid(&self) -> bool {
        !self.tag_name.is_empty()
    }


    /// Check if the tag is the current tag or the parent tag of the current tag
    ///
    /// # Arguments
    /// * `tag_to_check` - A reference to the `GameplayTag` that needs to be checked against the current object's tags.
    /// * `tags_manager` - A resource reference to the `GameplayTagsManager`, which is used to manage and query gameplay tags.
    ///
    /// # Returns
    /// * `bool` - Returns `true` if the current object contains the `tag_to_check`, otherwise returns `false`.
    ///
    /// # Errors
    /// This function does not return any errors directly. However, it assumes that the `tags_manager` is correctly initialized and can provide a valid `GameplayTagContainer` for the current object. If the `tags_manager` cannot provide a container (returns `None`), the function will return `false`.
    ///
    /// # Examples
    /// ```rust
    /// // Assuming `self` is an instance of a struct that uses this method,
    /// // `tag_to_check` is a `GameplayTag` you want to check,
    /// // and `tags_manager` is a properly initialized `Res<GameplayTagsManager>`.
    /// let result = self.matches_tag(&tag_to_check, &tags_manager);
    /// assert_eq!(result, true);  // Or false, depending on whether the tag is present.
    /// ```
    ///
    pub fn matches_tag(
        &self,
        tag_to_check: &GameplayTag,
        tags_manager: &Res<GameplayTagsManager>,
    ) -> bool {
        let complete_container = tags_manager.get_single_tag_container(self);
        if let Some(exist_container) = complete_container {
            exist_container.has_tag(tag_to_check)
        } else {
            false
        }
    }


    /// Checks if the current tag exactly matches the provided `GameplayTag`.
    ///
    /// # Arguments
    ///
    /// * `tag_to_check` - A reference to the `GameplayTag` to compare against.
    ///
    /// # Returns
    ///
    /// * `bool` - `true` if both tags are valid and their names match exactly, `false` otherwise.
    ///
    /// # Examples
    ///
    /// ```
    /// # use your_gameplay_tag_module::GameplayTag; // Replace with actual module path
    /// let tag1 = GameplayTag::new("Some.Tag");
    /// let tag2 = GameplayTag::new("Some.Tag");
    /// assert!(tag1.matches_tag_exact(&tag2));
    /// ```
    /// ```
    pub fn matches_tag_exact(&self, tag_to_check: &GameplayTag) -> bool {
        if !tag_to_check.is_valid() {
            false
        } else {
            self.tag_name == tag_to_check.tag_name
        }
    }


    ///
    /// Determine whether `container_to_check` contains the current Tag or its parent Tag
    ///
    /// # Arguments
    ///
    /// * `container_to_check` - A reference to a `GameplayTagContainer` whose tags are checked against the current object's tags.
    /// * `tags_manager` - A resource reference to the `GameplayTagsManager`, used for resolving the full tag container of the current object.
    ///
    /// # Returns
    ///
    /// * `true` if at least one tag from `container_to_check` is found in the resolved tag container of the current object.
    /// * `false` otherwise, or if the current object does not have an associated tag container.
    ///
    /// # Examples
    ///
    /// ```
    /// // Assuming `self` has a tag "Ability.Fire", and `container_to_check` contains "Ability.Fire" and "Ability.Ice"
    /// let result = self.matches_any(&container_to_check, &tags_manager);
    /// assert_eq!(result, true);
    /// ```
    ///
    pub fn matches_any(
        &self,
        container_to_check: &GameplayTagContainer,
        tags_manager: &Res<GameplayTagsManager>,
    ) -> bool {
        let complete_container = tags_manager.get_single_tag_container(self);
        if let Some(exist_container) = complete_container {
            exist_container.has_any(container_to_check)
        } else {
            false
        }
    }


    ///
    /// Checks if the current tag is exactly present in the given `GameplayTagContainer`.
    ///
    /// # Arguments
    ///
    /// * `container_to_check` - A reference to a `GameplayTagContainer` to search within.
    ///
    /// # Returns
    ///
    /// * `bool` - `true` if the current tag is found exactly in the container, otherwise `false`.
    ///
    /// # Examples
    ///
    /// ```
    /// # use your_crate::GameplayTag;
    /// # use your_crate::GameplayTagContainer;
    /// let tag = GameplayTag::new("Some.Tag");
    /// let mut container = GameplayTagContainer::new();
    /// container.add_tag(GameplayTag::new("Some.Tag"));
    /// assert!(tag.matches_any_exact(&container));
    /// ```
    ///
    /// This method uses binary search for efficient lookup, which requires the `GameplayTagContainer`'s
    /// internal list of tags to be sorted. If the `container_to_check` is empty, it returns `false`
    /// immediately.
    pub fn matches_any_exact(&self, container_to_check: &GameplayTagContainer) -> bool {
        if container_to_check.is_empty() {
            false
        } else {
            container_to_check.gameplay_tags.binary_search(self).is_ok()
        }
    }
}
