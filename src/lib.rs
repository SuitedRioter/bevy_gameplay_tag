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
//! use bevy_gameplay_tag::{GameplayTagCountContainer, GameplayTagsPlugin};
//!
//! fn main() {
//!     App::new()
//!         .add_plugins(DefaultPlugins)
//!         .add_plugins(GameplayTagsPlugin::with_data_path("assets/tag_data.json"))
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
//! ## Typed tag helpers
//!
//! ```rust
//! use bevy_gameplay_tag::{gameplay_tag, GameplayTag};
//!
//! mod tags {
//!     use bevy_gameplay_tag::gameplay_tag_names;
//!
//!     gameplay_tag_names! {
//!         pub DAMAGED = "Status.Damaged";
//!         pub BUFF_STRENGTH = "Buff.Strength";
//!     }
//! }
//!
//! let damaged: GameplayTag = gameplay_tag!(tags::DAMAGED);
//! assert_eq!(damaged.as_str(), "Status.Damaged");
//! ```
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

/// Build a [`GameplayTag`] from a string expression.
///
/// This is a lightweight convenience macro for places where you want clearer call sites
/// without spelling `GameplayTag::new(...)` repeatedly.
#[macro_export]
macro_rules! gameplay_tag {
    ($name:expr) => {
        $crate::GameplayTag::new($name)
    };
}

/// Define a group of gameplay tag name constants.
///
/// Use this inside a `mod tags { ... }` block to keep tag strings centralized and easy to refactor.
#[macro_export]
macro_rules! gameplay_tag_names {
    ($( $vis:vis $name:ident = $value:literal; )+ $(,)?) => {
        $(
            $vis const $name: &str = $value;
        )+
    };
}

// Re-export commonly used types
pub use gameplay_tag::{GameplayTag, InvalidTagName};
pub use gameplay_tag_container::{
    GameplayTagContainer, GameplayTagQuery, GameplayTagQueryExpression,
};
pub use gameplay_tag_count_container::{
    GameplayTagCountContainer, GameplayTagEventType, OnGameplayEffectTagCountChanged,
};
pub use gameplay_tag_requirements::GameplayTagRequirements;
pub use gameplay_tags_manager::{GameplayTagTableRow, GameplayTagsLoadError, GameplayTagsManager, GameplayTagsSettings};
pub use gameplay_tags_plugin::GameplayTagsPlugin;
