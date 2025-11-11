use crate::gameplay_tags_manager::{GameplayTagsManager, GameplayTagsSettings};
use bevy::app::{App, Plugin};

pub struct GameplayTagsPlugin{
    pub data_path: Option<String>,
}

impl Plugin for GameplayTagsPlugin {
    fn build(&self, app: &mut App) {
        if let Some(data_path) = &self.data_path {
            app.insert_resource(GameplayTagsSettings::with_data_path(data_path.clone()));
        }else { 
            app.insert_resource(GameplayTagsSettings::default());
        }
        app.init_resource::<GameplayTagsManager>();
    }
}

impl GameplayTagsPlugin {
    pub fn new() -> Self {
        Self {
            data_path: None,
        }
    }
    
    pub fn with_data_path(data_path: String) -> Self {
        GameplayTagsPlugin {
            data_path: Some(data_path),
        }
    }
}
