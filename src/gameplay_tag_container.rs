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

    ///
    /// Determines if the current container has a specific gameplay tag.
    /// Includes checking explicit and implicit tags for current container.
    ///
    /// # Arguments
    /// * `tag` - A reference to the `GameplayTag` to search for.
    ///
    /// # Returns
    /// * `true` if the tag is found in either the `gameplay_tags` or `parent_tags` collections.
    /// * `false` otherwise.
    ///
    /// This function performs a binary search on both the `gameplay_tags` and `parent_tags` collections,
    /// checking for the presence of the given tag. It returns `true` as soon as the tag is found in one of the collections.
    ///
    pub fn has_tag(&self, tag: &GameplayTag) -> bool {
        self.gameplay_tags.binary_search(tag).is_ok() || self.parent_tags.binary_search(tag).is_ok()
    }

    ///
    /// Determine whether the explicit tag of the current container has a specific game tag.
    ///
    /// # Arguments
    /// * `tag` - A reference to the `GameplayTag` to search for.
    ///
    /// # Returns
    /// * `true` if the tag is found, otherwise `false`.
    ///
    /// This method uses binary search to efficiently find the tag, which requires that
    /// the `gameplay_tags` collection is already sorted. If the tag exists, it returns `true`,
    /// indicating an exact match was found. Otherwise, it returns `false`.
    ///
    pub fn has_tag_exact(&self, tag: &GameplayTag) -> bool {
        self.gameplay_tags.binary_search(tag).is_ok()
    }

    ///
    /// Check whether any explicit tags in the provided `container_to_check` exist in the current container.
    /// That is, it determines whether there is an intersection between the explicit tags of two containers.
    ///
    /// # Arguments
    ///
    /// * `container_to_check` - A reference to a `GameplayTagContainer` whose tags will be checked against the current container.
    ///
    /// # Returns
    ///
    /// * `bool` - `true` if at least one tag from `container_to_check` is found in the current container, otherwise `false`.
    ///
    /// # Examples
    ///
    /// ```
    /// # use your_crate::GameplayTagContainer; // Replace with actual path
    /// let container = GameplayTagContainer::new(vec!["Tag1", "Tag2"]);
    /// let other_container = GameplayTagContainer::new(vec!["Tag2", "Tag3"]);
    /// assert_eq!(container.has_any(&other_container), true);
    /// ```
    ///
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

    ///
    /// Checks if any  explicit tags in the provided `container_to_check` exactly match any explicit or implicit tags in the current container.
    ///
    /// # Arguments
    ///
    /// * `container_to_check` - A reference to a `GameplayTagContainer` whose tags will be checked against the current container.
    ///
    /// # Returns
    ///
    /// * `bool` - `true` if at least one tag from `container_to_check` exactly matches a tag in the current container, otherwise `false`.
    ///
    /// # Examples
    ///
    /// ```
    /// use your_gameplay_tag_module::GameplayTagContainer; // Replace with actual module and import
    ///
    /// let container1 = GameplayTagContainer::new(vec!["Tag1".into(), "Tag2".into()]);
    /// let container2 = GameplayTagContainer::new(vec!["Tag2".into(), "Tag3".into()]);
    /// assert_eq!(container1.has_any_exact(&container2), true);
    /// ```
    ///
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

    ///
    /// Checks if all the explicit tags in the provided `container_to_check` exactly match any explicit or implicit tags in the current container.
    ///
    /// # Arguments
    ///
    /// * `container_to_check` - A reference to a `GameplayTagContainer` whose tags will be checked against the current container.
    ///
    /// # Returns
    ///
    /// * `bool` - `true` if all of the tags from `container_to_check` are found in the current container, or if `container_to_check` is empty. `false` otherwise.
    ///
    /// # Examples
    ///
    /// ```
    /// # use your_crate_name::GameplayTagContainer; // Replace with actual crate and struct names
    /// let mut container = GameplayTagContainer::new();
    /// container.add_tag("tag1");
    /// container.add_tag("tag2");
    ///
    /// let check_container = GameplayTagContainer::from_tags(vec!["tag1", "tag2"]);
    /// assert!(container.has_all(&check_container));
    ///
    /// let check_container_with_extra = GameplayTagContainer::from_tags
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

    ///
    /// Checks if all the explicit tags in the provided `container_to_check` exactly match any explicit tags in the current container.
    ///
    /// # Arguments
    ///
    /// * `container_to_check` - A reference to the `GameplayTagContainer` whose tags are to be checked against the current container.
    ///
    /// # Returns
    ///
    /// * `bool` - `true` if all the tags in `container_to_check` are exactly present in the current container, or if `container_to_check` is empty. Otherwise, returns `false`.
    ///
    /// # Examples
    ///
    /// ```
    /// # use your_gameplay_tag_module::GameplayTagContainer; // Replace with actual module path
    /// let mut container1 = GameplayTagContainer::new();
    /// container1.add_tag("Tag1");
    /// container1.add_tag("Tag2");
    ///
    /// let mut container2 = GameplayTagContainer::new();
    /// container2.add_tag("Tag1");
    ///
    /// assert_eq!(container1.has_all_exact(&container2), true);
    ///
    /// container2.add_tag("Tag3");
    /// assert_eq!(container1.has_all_exact(&container2), false);
    /// ```
    ///
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

    ///
    /// Fills the `parent_tags` field of the current object with the parent tags
    /// associated with each tag in `gameplay_tags`. This method ensures that
    /// only unique parent tags are added, and it maintains the order of the
    /// `parent_tags` list.
    ///
    /// # Arguments
    ///
    /// * `tags_manager` - A reference to a resource of type `GameplayTagsManager`
    ///   which is used to fetch the complete tag container for each tag. The
    ///   `GameplayTagsManager` must be able to provide a tag's full details,
    ///   including its parent tags.
    ///
    /// # Behavior
    ///
    /// - Clears the existing `parent_tags` before adding new ones.
    /// - Iterates over each tag in `gameplay_tags`.
    /// - For each tag, retrieves its complete tag container from `tags_manager`.
    /// - If the tag container exists, iterates over its `parent_tags` and attempts
    ///   to insert each into the `parent_tags` of the current object.
    /// - Uses binary search to find the correct insertion point for each parent tag
    ///   to maintain sorted order, and inserts the tag if it is not already present.
    ///
    /// # Examples
    ///
    /// ```
    /// // Assuming `self` has some gameplay tags and `tags_manager` is properly set up.
    /// self.fill_parent_tags(&tags_manager);
    /// // After calling this method, `self.parent_tags` will contain all unique
    /// // parent tags from the `gameplay_tags` in a sorted manner.
    /// ```
    ///
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

    ///
    /// If the explicit tags from `other_a` match any explicit or implicit tags in `other_b`, they will be attached to the current container.
    ///
    /// # Arguments
    /// * `other_a` - A reference to the first `GameplayTagContainer` whose tags are to be appended.
    /// * `other_b` - A reference to the second `GameplayTagContainer` used for matching against `other_a`'s tags.
    /// * `tags_manager` - A resource reference to the `GameplayTagsManager` used for tag operations.
    ///
    /// # Details
    /// This method iterates over each tag in `other_a`. For each tag, it checks if there is a match
    /// with any of the tags in `other_b` using the `matches_any` method. If a match is found, the tag
    /// from `other_a` is cloned and added to the current `GameplayTagContainer` instance using the
    /// `add_tag` method. The `tags_manager` is required for performing tag-related operations such as
    /// adding or checking matches.
    ///
    /// # Examples
    /// ```
    /// // Assuming `self`, `other_a`, `other_b`, and `tags_manager` are properly initialized
    /// self.append_matches_tags(&other_a, &other_b, &tags_manager);
    /// ```
    ///
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

    ///
    /// Appends all the explicit tags from another `GameplayTagContainer` to the current container.
    ///
    /// # Arguments
    ///
    /// * `other` - A reference to the `GameplayTagContainer` from which tags will be appended.
    /// * `tags_manager` - A resource reference to the `GameplayTagsManager` used for managing tag operations.
    ///
    /// This function iterates over each tag in the `other` container and adds it to the current container,
    /// using the provided `tags_manager` to handle the addition.
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

    ///
    /// Returns all labels of the container, including display labels and implicit labels.
    /// Note that only the gameplay_tags attribute of the returned container has a value, that is, all tags are placed in the gameplay_tags attribute.
    ///
    /// # Returns
    /// * A new `GameplayTagContainer` containing the original gameplay tags and their parent tags.
    ///
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

    fn find_tag_index(&self, tag: &GameplayTag) -> Option<usize> {
        self.gameplay_tags.binary_search(tag).ok()
    }
}
