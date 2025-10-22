use crate::gameplay_tags_manager::GameplayTagsManager;
use bevy::app::{App, Plugin};

pub struct GameplayTagsPlugin;

impl Plugin for GameplayTagsPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<GameplayTagsManager>();
    }
}
