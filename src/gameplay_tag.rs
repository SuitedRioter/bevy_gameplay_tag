use std::fmt::{Debug, Display};
use std::hash::{Hash, Hasher};

use string_cache::DefaultAtom as FName;

use crate::{
    gameplay_tag_container::GameplayTagContainer, gameplay_tags_manager::GameplayTagsManager,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InvalidTagName {
    name: String,
    reason: &'static str,
}

impl InvalidTagName {
    fn new(name: &str, reason: &'static str) -> Self {
        Self {
            name: name.to_string(),
            reason,
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn reason(&self) -> &'static str {
        self.reason
    }
}

impl Display for InvalidTagName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "invalid gameplay tag name '{}': {}", self.name, self.reason)
    }
}

impl std::error::Error for InvalidTagName {}

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

impl Display for GameplayTag {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

impl From<&str> for GameplayTag {
    fn from(value: &str) -> Self {
        Self::new(value)
    }
}

impl From<String> for GameplayTag {
    fn from(value: String) -> Self {
        Self::new(&value)
    }
}

impl AsRef<str> for GameplayTag {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl GameplayTag {
    pub fn new(full_name: &str) -> GameplayTag {
        GameplayTag {
            tag_name: FName::from(full_name),
        }
    }

    pub fn try_new(full_name: &str) -> Result<GameplayTag, InvalidTagName> {
        if full_name.is_empty() {
            return Err(InvalidTagName::new(full_name, "tag name cannot be empty"));
        }

        if full_name.starts_with('.') || full_name.ends_with('.') {
            return Err(InvalidTagName::new(
                full_name,
                "tag name cannot start or end with '.'",
            ));
        }

        if full_name.contains("..") {
            return Err(InvalidTagName::new(
                full_name,
                "tag name cannot contain consecutive '.' separators",
            ));
        }

        for segment in full_name.split('.') {
            if segment.is_empty() {
                return Err(InvalidTagName::new(
                    full_name,
                    "tag name cannot contain empty segments",
                ));
            }

            if !segment
                .chars()
                .all(|ch| ch.is_ascii_alphanumeric() || ch == '_')
            {
                return Err(InvalidTagName::new(
                    full_name,
                    "each tag segment must contain only letters, numbers, or underscores",
                ));
            }
        }

        Ok(Self::new(full_name))
    }

    pub fn get_tag_name(&self) -> &str {
        self.as_str()
    }

    pub fn as_str(&self) -> &str {
        &self.tag_name
    }

    pub fn name(&self) -> &str {
        self.as_str()
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
    /// ```ignore
    /// // Requires a populated GameplayTagsManager.
    /// let fire = GameplayTag::new("Ability.Skill.Fire");
    /// let parent = GameplayTag::new("Ability.Skill");
    /// assert!(fire.matches_tag(&parent, &tags_manager));
    /// ```
    ///
    pub fn matches_tag(
        &self,
        tag_to_check: &GameplayTag,
        tags_manager: &GameplayTagsManager,
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
    /// ```rust
    /// use bevy_gameplay_tag::GameplayTag;
    ///
    /// let tag1 = GameplayTag::new("Some.Tag");
    /// let tag2 = GameplayTag::new("Some.Tag");
    /// assert!(tag1.matches_tag_exact(&tag2));
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
    /// ```ignore
    /// // Requires a populated GameplayTagsManager and a container built with the same tags.
    /// let fire = GameplayTag::new("Ability.Skill.Fire");
    /// let result = fire.matches_any(&container_to_check, &tags_manager);
    /// assert!(result);
    /// ```
    ///
    pub fn matches_any(
        &self,
        container_to_check: &GameplayTagContainer,
        tags_manager: &GameplayTagsManager,
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
    /// ```ignore
    /// // Build the container with GameplayTagContainer::add_tag before checking.
    /// let tag = GameplayTag::new("Some.Tag");
    /// assert!(tag.matches_any_exact(&container_to_check));
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

#[cfg(test)]
mod tests {
    use super::GameplayTag;

    #[test]
    fn try_new_accepts_valid_hierarchical_tags() {
        let tag = GameplayTag::try_new("Ability.Skill.Fire").unwrap();
        assert_eq!(tag.as_str(), "Ability.Skill.Fire");
    }

    #[test]
    fn try_new_rejects_empty_names() {
        assert!(GameplayTag::try_new("").is_err());
    }

    #[test]
    fn try_new_rejects_edge_dots() {
        assert!(GameplayTag::try_new(".Ability").is_err());
        assert!(GameplayTag::try_new("Ability.").is_err());
    }

    #[test]
    fn try_new_rejects_consecutive_dots() {
        assert!(GameplayTag::try_new("Ability..Fire").is_err());
    }

    #[test]
    fn try_new_rejects_invalid_segment_characters() {
        assert!(GameplayTag::try_new("Ability.Skill-Fire").is_err());
        assert!(GameplayTag::try_new("Ability.Skill Fire").is_err());
    }
}
