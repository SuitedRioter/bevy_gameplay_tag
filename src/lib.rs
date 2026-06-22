//! # Bevy Gameplay Tag
//!
//! A powerful and flexible hierarchical gameplay tag system for the Bevy game engine,
//! inspired by Unreal Engine's Gameplay Tag system.
//!
//! ## Features
//!
//! - **Hierarchical Tag System**: Create parent-child tag relationships (e.g., `Ability.Skill.Fire`)
//! - **Flexible Matching**: Support for both exact and hierarchical tag matching
//! - **Reference Counting**: Track tag counts with automatic event notifications
//! - **Complex Queries**: Build sophisticated tag queries with boolean logic
//! - **Event-Driven**: Observer pattern for responding to tag changes
//! - **JSON Configuration**: Define your tag hierarchy in external JSON files
//! - **High Performance**: Optimized with string interning and binary search
//!
//! ## Quick Start
//!
//! ```rust,no_run
//! use bevy::prelude::*;
//! use bevy_gameplay_tag::{GameplayTagsPlugin, GameplayTag, GameplayTagCountContainer};
//!
//! fn main() {
//!     App::new()
//!         .add_plugins(DefaultPlugins)
//!         .add_plugins(GameplayTagsPlugin::default())
//!         .add_systems(Startup, setup)
//!         .run();
//! }
//!
//! fn setup(mut commands: Commands) {
//!     commands.spawn(GameplayTagCountContainer::new());
//! }
//! ```
//!
//! ## Core Types
//!
//! - [`GameplayTag`]: Immutable tag identifier with hierarchical matching
//! - [`GameplayTagContainer`]: Collection of tags with query capabilities
//! - [`GameplayTagCountContainer`]: Reference-counted tags with event notifications
//! - [`GameplayTagsManager`]: Global resource managing the tag hierarchy
//! - [`GameplayTagsPlugin`]: Bevy plugin for initialization
//!
//! ## Examples
//!
//! See the [examples directory](https://github.com/SuitedRioter/bevy_gameplay_tag/tree/main/examples)
//! for complete working examples.

pub mod gameplay_tag;
pub mod gameplay_tag_container;
pub mod gameplay_tag_count_container;
pub mod gameplay_tag_requirements;
pub mod gameplay_tags_manager;
pub mod gameplay_tags_plugin;

// Re-export commonly used types
pub use gameplay_tag::GameplayTag;
pub use gameplay_tag_container::{
    GameplayTagContainer, GameplayTagQuery, GameplayTagQueryExpression,
};
pub use gameplay_tag_count_container::{
    GameplayTagCountContainer, GameplayTagEventType, OnGameplayEffectTagCountChanged,
};
pub use gameplay_tag_requirements::GameplayTagRequirements;
pub use gameplay_tags_manager::{GameplayTagsManager, GameplayTagsSettings};
pub use gameplay_tags_plugin::GameplayTagsPlugin;
