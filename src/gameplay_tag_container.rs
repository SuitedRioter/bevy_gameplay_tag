use crate::gameplay_tag::GameplayTag;
use bevy::prelude::Component;

#[derive(Component)]
pub struct GameplayTagContainer {
    pub gameplay_tags: Vec<GameplayTag>,
    pub parent_tags: Vec<GameplayTag>,
}

impl GameplayTagContainer {
    pub fn new() -> Self {
        GameplayTagContainer {
            gameplay_tags: Vec::new(),
            parent_tags: Vec::new(),
        }
    }
}
