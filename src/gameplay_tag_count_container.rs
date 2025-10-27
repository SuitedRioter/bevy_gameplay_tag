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
    ///所有标签的计数，包括父标签，比如添加A.B,这里就不仅A.B计数+1，父标签A也会+1
    gameplay_tag_count_map: HashMap<GameplayTag, i32>,
    ///显示标签计数，只添加标签本身计数，不包括父标签。比如添加A.B,这里就只有A.B计数+1
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

    pub fn has_matching_gameplay_tag(&self, tag_to_check: &GameplayTag) -> bool {
        let count = self.gameplay_tag_count_map.get(tag_to_check).copied();
        count.is_some() && count.unwrap() > 0
    }

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

    pub fn get_tag_count(&self, tag: &GameplayTag) -> i32 {
        if let Some(count) = self.gameplay_tag_count_map.get(tag) {
            *count
        } else {
            0
        }
    }

    #[inline]
    pub fn get_explicit_tag_count(&self, tag: &GameplayTag) -> i32 {
        if let Some(count) = self.explicit_tag_count_map.get(tag) {
            *count
        } else {
            0
        }
    }

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

    /// 这里原本有3个事件触发场景
    ///
    /// 1. 当容器中任何Tag发生"存在性变化"（从无到有或从有到无）时触发。它适用于需要感知整体状态变化的场景，而不关心具体是哪个Tag发生了变化。
    ///    观察者直接监听这个实体的OnGameplayEffectTagCountChanged事件即可，不关心是内部是哪个tag，只需要关心event_type
    /// 2. 这个容器内，某个特定标签的2种通知，就是EventType的那两种，在特定时刻去发送。
    ///
    ///
    /// 虚幻的GameplayTagCountContainer::gather_tag_change_delegates在这里只是定义了委托，并没有发送事件.
    /// 不过bevy特性，不需要这么做，bevy的事件触发后，会先放缓冲队列，然后在这个system结束后，立刻执行观察者代码，近似等同于汇总后，再集体触发。
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
