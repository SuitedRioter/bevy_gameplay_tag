use bevy::{
    ecs::{
        component::Component,
        entity::Entity,
        event::EntityEvent,
        observer::{ObservedBy, Observer},
        system::{Commands, Res},
        world::World,
    },
    log::warn,
    platform::collections::HashMap,
};

use crate::{
    gameplay_tag::GameplayTag, gameplay_tag_container::GameplayTagContainer,
    gameplay_tags_manager::GameplayTagsManager,
};

#[derive(Component, Debug)]
pub struct GameplayTagCountContainer {
    //所有标签的计数，包括父标签，比如添加A.B,这里就不仅A.B计数+1，父标签A也会+1
    gameplay_tag_count_map: HashMap<GameplayTag, i32>,
    //显示标签计数，只添加标签本身计数，不包括父标签。比如添加A.B,这里就只有A.B计数+1
    explicit_tag_count_map: HashMap<GameplayTag, i32>,
    explicit_tags: GameplayTagContainer,
}

impl GameplayTagCountContainer {
    pub fn new() -> Self {
        Self {
            gameplay_tag_count_map: HashMap::new(),
            explicit_tag_count_map: HashMap::new(),
            explicit_tags: GameplayTagContainer::new(),
        }
    }

    ///
    /// Checks if the current object has a specific gameplay tag.
    ///
    /// # Arguments
    ///
    /// * `tag_to_check` - A reference to the `GameplayTag` to check for in the current object's tag collection.
    ///
    /// # Returns
    ///
    /// * `bool` - `true` if the tag is present and its count is greater than 0, otherwise `false`.
    ///
    /// # Examples
    ///
    /// ```
    /// # use your_crate::GameplayTag;
    /// # use your_crate::YourStruct; // Replace YourStruct with the actual struct name that implements this method
    /// let obj = YourStruct::new();
    /// let tag = GameplayTag::from("example_tag");
    /// assert_eq!(obj.has_matching_gameplay_tag(&tag), true);
    /// ```
    ///
    /// This function looks up the given `tag_to_check` in the internal `gameplay_tag_count_map` of the object. It returns `true`
    /// only if the tag exists in the map and its associated count is more than zero, indicating that the tag is indeed
    /// active or present on the object.
    ///
    #[inline]
    pub fn has_matching_gameplay_tag(&self, tag_to_check: &GameplayTag) -> bool {
        let count = self.gameplay_tag_count_map.get(tag_to_check).copied();
        count.is_some() && count.unwrap() > 0
    }

    ///
    /// Checks if all the gameplay tags in the provided `tag_container` are present and have a non-zero count in the current object's tag count map.
    ///
    /// # Arguments
    /// * `tag_container` - A reference to a `GameplayTagContainer` that contains the gameplay tags to check.
    ///
    /// # Returns
    /// * `true` if all of the tags in `tag_container` are present in the current object's tag count map with a non-zero count, otherwise `false`.
    ///
    /// # Notes
    /// * If `tag_container` is empty, the function immediately returns `false`.
    /// * The function iterates over each tag in `tag_container`, checking for its presence and a non-zero count in the current object's tag count map. If any tag is missing or has a zero count, it returns `false`.
    /// * If all tags are found with a non-zero count, the function returns `true`.
    #[inline]
    pub fn has_all_matching_gameplay_tags(&self, tag_container: &GameplayTagContainer) -> bool {
        if tag_container.is_empty() {
            return false;
        }

        for tag in tag_container.gameplay_tags.iter() {
            let count = self.gameplay_tag_count_map.get(tag).copied();
            if count.is_none() || count.unwrap() == 0 {
                return false;
            }
        }

        true
    }

    ///
    /// Determines if the current object has any gameplay tags that match those in the provided `tag_container`.
    ///
    /// # Arguments
    /// * `tag_container` - A reference to a `GameplayTagContainer` containing the tags to check against.
    ///
    /// # Returns
    /// * `true` if at least one tag from `tag_container` is found in the current object's tag collection and its count is greater than 0, otherwise `false`.
    ///
    /// # Notes
    /// - If the `tag_container` is empty, the function immediately returns `false`.
    /// - The function iterates over each tag in the `tag_container`, checking for its presence and positive count in the current object's `gameplay_tag_count_map`.
    /// - The search stops as soon as a matching tag with a count greater than 0 is found.
    #[inline]
    pub fn has_any_matching_gameplay_tags(self, tag_container: &GameplayTagContainer) -> bool {
        if tag_container.is_empty() {
            return false;
        }

        for tag in tag_container.gameplay_tags.iter() {
            let count = self.gameplay_tag_count_map.get(tag).copied();
            if count.is_some() && count.unwrap() > 0 {
                return true;
            }
        }

        false
    }

    ///
    /// Updates the count of each tag in the provided `GameplayTagContainer` by a specified delta.
    /// This function ensures that the internal state of tags is correctly updated, including
    /// adjustments to parent tags when necessary. It handles both increment and decrement operations,
    /// with special handling for decrement operations to ensure proper maintenance of parent-child
    /// relationships among tags.
    ///
    /// # Arguments
    /// * `container` - A reference to the `GameplayTagContainer` whose tags' counts are to be updated.
    /// * `count_delta` - The amount by which to update the count of each tag in the container. Can be positive or negative.
    /// * `tags_manager` - A resource containing the `GameplayTagsManager` used to manage gameplay tags, including their parent-child relationships.
    /// * `commands` - A mutable reference to the `Commands` struct used to queue commands for later execution within the Bevy ECS (Entity-Component-System) framework.
    /// * `entity` - The `Entity` associated with the tag updates. Used to identify the entity for which the tag map is being updated.
    ///
    /// # Behavior
    /// - If `count_delta` is not zero, the function iterates over each tag in the `container`.
    /// - For each tag, it calls `update_tag_map_deferred_parent_removal_internal` to adjust the tag's count.
    /// - If any tag's count is updated and `count_delta` is negative, indicating a removal, it will also call `fill_parent_tags` on `explicit_tags` to ensure parent tags are correctly updated after child tags have been potentially removed.
    /// - If `count_delta` is positive, indicating an addition, no further action is required as adding tags automatically includes their parent tags.
    #[inline]
    pub fn update_tag_container_count(
        &mut self,
        container: &GameplayTagContainer,
        count_delta: i32,
        tags_manager: &Res<GameplayTagsManager>,
        commands: &mut Commands,
        entity: Entity,
    ) {
        if count_delta != 0 {
            let mut updated_any = false;
            for tag in container.gameplay_tags.iter() {
                updated_any |= self.update_tag_map_deferred_parent_removal_internal(
                    tag,
                    count_delta,
                    tags_manager,
                    commands,
                    entity,
                )
            }
            //因为如果是减少，则有可能某个标签为0被删除，而上面update的里面的remove_tag默认使用的是延迟重建父级（defer_parent_tags_on_remove），所以这里要更新父级。
            //如果是增加，上面update的里面的add_tag会自动添加父级，所以不用管
            if updated_any && count_delta < 0 {
                self.explicit_tags.fill_parent_tags(tags_manager);
            }
        }
    }

    ///
    /// Updates the count of a specified gameplay tag for a given entity.
    ///
    /// # Arguments
    ///
    /// * `tag` - A reference to the `GameplayTag` to be updated.
    /// * `count_delta` - The change in count to apply to the tag. If this is 0, the function returns `false` without making any changes.
    /// * `tags_manager` - A resource reference to the `GameplayTagsManager` that manages all gameplay tags.
    /// * `commands` - A mutable reference to `Commands` used to queue commands for the Bevy ECS (Entity Component System).
    /// * `entity` - The `Entity` for which the tag count is being updated.
    ///
    /// # Returns
    ///
    /// * `true` if the tag count was successfully updated and the operation resulted in a non-zero delta.
    /// * `false` if `count_delta` is 0, indicating no update was necessary or performed.
    ///
    /// This method checks if the `count_delta` is not zero before attempting to update the tag count. If an update is needed, it calls `update_tag_map_internal` to perform the actual update. Otherwise, it returns `false` immediately.
    #[inline]
    pub fn update_tag_count(
        &mut self,
        tag: &GameplayTag,
        count_delta: i32,
        tags_manager: &Res<GameplayTagsManager>,
        commands: &mut Commands,
        entity: Entity,
    ) -> bool {
        if count_delta != 0 {
            self.update_tag_map_internal(tag, count_delta, tags_manager, commands, entity)
        } else {
            false
        }
    }

    ///
    /// Updates the count of a specific gameplay tag, deferring the removal of parent tags if necessary.
    ///
    /// # Arguments
    ///
    /// * `tag` - A reference to the `GameplayTag` whose count is to be updated.
    /// * `count_delta` - The change in count for the specified tag. Positive values increase the count, while negative values decrease it.
    /// * `tags_manager` - A resource reference to the `GameplayTagsManager` that manages all gameplay tags and their relationships.
    /// * `commands` - A mutable reference to `Commands` used to queue commands for entity modification.
    /// * `entity` - The `Entity` to which the tag count update applies.
    ///
    /// # Returns
    ///
    /// * `bool` - Returns `true` if the tag's count was successfully updated and any required parent tag removals were deferred. Returns `false` if no update was made (e.g., when `count_delta` is 0).
    ///
    /// # Notes
    ///
    /// - This function does not immediately remove tags from the entity. Instead, it defers the removal of parent tags based on the new count, allowing for more complex tag management scenarios.
    /// - If `count_delta` is 0, the function returns `false` without making any changes, as there is no need to update the tag count or defer any operations.
    ///
    #[inline]
    pub fn update_tag_count_deferred_parent_removal(
        &mut self,
        tag: &GameplayTag,
        count_delta: i32,
        tags_manager: &Res<GameplayTagsManager>,
        commands: &mut Commands,
        entity: Entity,
    ) -> bool {
        if count_delta != 0 {
            self.update_tag_map_deferred_parent_removal_internal(
                tag,
                count_delta,
                tags_manager,
                commands,
                entity,
            )
        } else {
            false
        }
    }

    ///
    /// Sets the count of a specific gameplay tag for an entity.
    ///
    /// # Arguments
    ///
    /// * `tag` - A reference to the `GameplayTag` whose count is to be set.
    /// * `new_count` - The new count for the specified tag.
    /// * `tags_manager` - A resource reference to the `GameplayTagsManager` used for managing tags.
    /// * `commands` - A mutable reference to `Commands` for executing commands on the world.
    /// * `entity` - The `Entity` for which the tag count is being set.
    ///
    /// # Returns
    ///
    /// * `bool` - Returns `true` if the tag count was updated (i.e., there was a change in the count), otherwise `false`.
    ///
    /// This function checks if the current count of the given tag differs from the new count. If it does, it updates the internal tag map and performs any necessary operations through the provided `Commands`. If the new count is the same as the existing one, no action is taken, and `false` is returned.
    ///
    #[inline]
    pub fn set_tag_count(
        &mut self,
        tag: &GameplayTag,
        new_count: i32,
        tags_manager: &Res<GameplayTagsManager>,
        commands: &mut Commands,
        entity: Entity,
    ) -> bool {
        let mut existing_count = 0;
        if let Some(count) = self.explicit_tag_count_map.get(tag) {
            existing_count = *count;
        }

        let count_delta = new_count - existing_count;
        if count_delta != 0 {
            self.update_tag_map_internal(tag, count_delta, tags_manager, commands, entity)
        } else {
            false
        }
    }

    ///
    /// Retrieves the count of a specific `GameplayTag` from the internal map.
    ///
    /// # Arguments
    ///
    /// * `tag` - A reference to the `GameplayTag` whose count is to be retrieved.
    ///
    /// # Returns
    ///
    /// * The count of the provided `GameplayTag` if it exists in the map, otherwise 0.
    ///
    /// # Examples
    ///
    /// ```
    /// let tag = GameplayTag::new("example_tag");
    /// let count = some_instance.get_tag_count(&tag);
    /// assert_eq!(count, 5); // Assuming "example_tag" has a count of 5 in the map.
    /// ```
    ///
    #[inline]
    pub fn get_tag_count(&self, tag: &GameplayTag) -> i32 {
        if let Some(count) = self.gameplay_tag_count_map.get(tag) {
            *count
        } else {
            0
        }
    }

    ///
    /// Retrieves the count of an explicit tag within the current context.
    ///
    /// # Arguments
    /// * `tag` - A reference to a `GameplayTag` for which the count is requested.
    ///
    /// # Returns
    /// * The count of the specified tag as `i32`. If the tag is not found, returns 0.
    ///
    /// # Examples
    /// ```
    /// let my_tag = GameplayTag::new("My.Tag");
    /// let count = some_object.get_explicit_tag_count(&my_tag);
    /// assert_eq!(count, 5); // Assuming "My.Tag" has been added 5 times
    /// ```
    ///
    #[inline]
    pub fn get_explicit_tag_count(&self, tag: &GameplayTag) -> i32 {
        if let Some(count) = self.explicit_tag_count_map.get(tag) {
            *count
        } else {
            0
        }
    }

    ///
    /// Resets the state of the current object and removes `Observer` components from all entities
    /// that are observing the specified entity.
    ///
    /// # Arguments
    ///
    /// * `world` - A mutable reference to the `World` in which the entity and its observers exist.
    /// * `entity` - The `Entity` for which the reset operation is being performed. This entity is
    ///   assumed to have an `ObservedBy` component that lists all entities observing it.
    ///
    /// # Effects
    ///
    /// - Clears the `explicit_tag_count_map` and `gameplay_tag_count_map`.
    /// - Resets the `explicit_tags`.
    /// - Iterates over all observer entities listed in the `ObservedBy` component of the given entity
    ///   and removes the `Observer` component from each, leaving the observer entities themselves intact.
    /// - If there's a need to remove the observer entities entirely, it's suggested to add a dedicated
    ///   observation marker component to these entities, which can then be checked and used as a basis
    ///   for removal.
    ///
    /// # Examples
    ///
    /// ```rust
    /// // Assuming `world` is a mutable reference to a World instance and `entity` is the target Entity.
    /// let mut some_object = SomeObject::new();
    /// some_object.reset(&mut world, entity);
    /// ```
    ///
    /// # Notes
    ///
    /// - This function does not remove the observer entities themselves; it only removes their `Observer`
    ///   components. For complete removal, additional logic must be implemented.
    ///
    pub fn reset(&mut self, world: &mut World, entity: Entity) {
        self.explicit_tag_count_map.clear();
        self.explicit_tags.reset();
        self.gameplay_tag_count_map.clear();
        if let Some(observed_by) = world.get::<ObservedBy>(entity) {
            let observer_entities: Vec<Entity> = observed_by.get().to_vec();
            for observer_entity in observer_entities {
                // 只移除 Observer 组件,保留观察者实体
                // 如果需要移除实体，建议给这个实体新增一个观察专用标记组件，这里就可以判断进行移除。
                world.entity_mut(observer_entity).remove::<Observer>();
            }
        }
    }

    ///
    /// Fills the parent tags for the explicit tags associated with the current object.
    ///
    /// This method iterates through the `explicit_tags` and for each tag, it retrieves
    /// and adds all its parent tags from the provided `tags_manager`. This ensures that
    /// the object not only contains the explicitly defined tags but also implicitly
    /// includes all the parent tags, expanding the tag coverage and utility for game logic,
    /// such as in conditions or queries.
    ///
    /// # Arguments
    ///
    /// * `tags_manager` - A reference to a resource of type `GameplayTagsManager` which
    ///   is responsible for managing and providing information about all available tags,
    ///   including their hierarchy.
    ///
    /// # Example
    ///
    /// ```
    /// // Assuming `self.explicit_tags` initially contains some specific tags.
    /// // After calling `fill_parent_tags`, `self.explicit_tags` will also include
    /// // all the parent tags of the initial set.
    /// self.fill_parent_tags(&tags_manager);
    /// ```
    ///
    pub fn fill_parent_tags(&mut self, tags_manager: &Res<GameplayTagsManager>) {
        self.explicit_tags.fill_parent_tags(tags_manager);
    }

    fn update_tag_map_internal(
        &mut self,
        tag: &GameplayTag,
        count_delta: i32,
        tags_manager: &Res<GameplayTagsManager>,
        commands: &mut Commands,
        entity: Entity,
    ) -> bool {
        if !self.update_explicit_tags(tag, count_delta, false, tags_manager) {
            false
        } else {
            self.gather_tag_change_delegates(tag, count_delta, tags_manager, commands, entity)
        }
    }

    fn update_tag_map_deferred_parent_removal_internal(
        &mut self,
        tag: &GameplayTag,
        count_delta: i32,
        tags_manager: &Res<GameplayTagsManager>,
        commands: &mut Commands,
        entity: Entity,
    ) -> bool {
        if !self.update_explicit_tags(tag, count_delta, true, tags_manager) {
            false
        } else {
            self.gather_tag_change_delegates(tag, count_delta, tags_manager, commands, entity)
        }
    }
    fn update_explicit_tags(
        &mut self,
        tag: &GameplayTag,
        count_delta: i32,
        defer_parent_tags_on_remove: bool,
        tags_manager: &Res<GameplayTagsManager>,
    ) -> bool {
        let tag_already_exists = self.explicit_tags.has_tag_exact(&tag);
        if !tag_already_exists {
            if count_delta > 0 {
                self.explicit_tags.add_tag(tag.clone(), tags_manager);
            } else {
                if self.explicit_tags.has_tag(&tag) {
                    warn!(
                        "试图从标记计数容器中删除标记：{}, 但该标记不在容器中！",
                        tag.get_tag_name()
                    );
                }
                return false;
            }
        }

        let existing_count = self.explicit_tag_count_map.entry(tag.clone()).or_insert(0);
        *existing_count = (*existing_count + count_delta).max(0);
        if *existing_count <= 0 {
            self.explicit_tags
                .remove_tag(&tag, defer_parent_tags_on_remove, tags_manager);
        }
        true
    }

    ///
    /// Gathers and processes tag change delegates for a given gameplay tag, updating the count of the tag,
    /// and triggering events based on the changes.
    ///
    /// # Arguments
    /// * `tag` - A reference to the `GameplayTag` that is being updated.
    /// * `count_delta` - The change in count for the specified tag. Positive values increase the count, negative values decrease it.
    /// * `tags_manager` - A resource containing the `GameplayTagsManager` which provides access to tag information including parent tags.
    /// * `commands` - A mutable reference to `Commands` used to trigger events.
    /// * `entity` - The `Entity` associated with the tag changes.
    ///
    /// # Returns
    /// * `bool` - Returns `true` if a significant change (addition or removal) occurred for any tag, otherwise `false`.
    ///
    /// This function updates the internal count of the specified tag and its parent tags, checks for significant changes (when a tag's count goes from non-zero to zero or vice versa),
    /// and triggers appropriate events (`OnGameplayEffectTagCountChanged`) with the type of event being either `NewOrRemoved` or `AnyCountChanged` based on the nature of the change.
    ///
    fn gather_tag_change_delegates(
        &mut self,
        tag: &GameplayTag,
        count_delta: i32,
        tags_manager: &Res<GameplayTagsManager>,
        commands: &mut Commands,
        entity: Entity,
    ) -> bool {
        let tag_and_parents_container = tags_manager.request_gameplay_tag_parents(tag);
        let mut created_significant_change = false;

        for tag in tag_and_parents_container.gameplay_tags.into_iter() {
            let tag_count = self.gameplay_tag_count_map.entry(tag.clone()).or_insert(0);
            let old_count = *tag_count;
            let new_count = (old_count + count_delta).max(0);
            *tag_count = new_count;
            //如果发生重大变化（新增或完全删除），触发相关事件
            let significant_change = old_count == 0 || new_count == 0;
            created_significant_change |= significant_change;
            if significant_change {
                //OnNewOrRemove
                commands.trigger(OnGameplayEffectTagCountChanged {
                    entity,
                    tag: tag.clone(),
                    new_count,
                    event_type: GameplayTagEventType::NewOrRemoved,
                });
            }
            //OnAnyChange
            commands.trigger(OnGameplayEffectTagCountChanged {
                entity,
                tag: tag.clone(),
                new_count,
                event_type: GameplayTagEventType::AnyCountChanged,
            });
        }

        created_significant_change
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum GameplayTagEventType {
    /** Event only happens when tag is new or completely removed */
    NewOrRemoved,
    /** Event happens any time tag "count" changes */
    AnyCountChanged,
}

/// 定义观察者函数
/// fn on_tag_changed(event: On<OnGameplayEffectTagCountChanged>) {
///     println!("标签 {:?} 计数变更为 {}",
///         event.event().tag,
///         event.event().new_count);
/// }
///
/// 使用函数名创建观察者
/// world.spawn_empty().observe(on_tag_changed);
#[derive(EntityEvent, Debug)]
#[allow(dead_code)]
pub struct OnGameplayEffectTagCountChanged {
    pub entity: Entity,
    pub tag: GameplayTag,
    pub new_count: i32,
    pub event_type: GameplayTagEventType,
}
