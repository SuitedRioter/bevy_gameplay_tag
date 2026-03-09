# Bevy Gameplay Tag

A powerful and flexible hierarchical gameplay tag system for the Bevy game engine, inspired by Unreal Engine's Gameplay Tag system.

[![License](https://img.shields.io/badge/license-MIT%2FApache-blue.svg)](LICENSE-MIT)
[![Bevy](https://img.shields.io/badge/Bevy-0.18-blue)](https://bevyengine.org)

[English](README.md) | [简体中文](README_zh.md)

## Features

- **Hierarchical Tag System**: Create parent-child tag relationships (e.g., `Ability.Skill.Fire`)
- **Flexible Matching**: Support for both exact and hierarchical tag matching
- **Reference Counting**: Track tag counts with automatic event notifications
- **Complex Queries**: Build sophisticated tag queries with boolean logic
- **Event-Driven**: Observer pattern for responding to tag changes
- **JSON Configuration**: Define your tag hierarchy in external JSON files
- **High Performance**: Optimized with string interning and binary search
- **Type Safe**: Leverages Rust's type system for compile-time safety

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
bevy_gameplay_tag = "0.1.0"
bevy = "0.18"
```

## Quick Start

### 1. Add the Plugin

```rust
use bevy::prelude::*;
use bevy_gameplay_tag::GameplayTagsPlugin;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(GameplayTagsPlugin::default())
        .run();
}
```

### 2. Define Your Tags (JSON)

Create a `tag_data.json` file:

```json
{
  "GameplayTagList": [
    {
      "Tag": "Ability",
      "DevComment": "Root tag for all abilities"
    },
    {
      "Tag": "Ability.Skill",
      "DevComment": "Skills subcategory"
    },
    {
      "Tag": "Ability.Skill.Fire",
      "DevComment": "Fire skill"
    },
    {
      "Tag": "Status.Buff",
      "DevComment": "Positive status effects"
    },
    {
      "Tag": "Status.Debuff",
      "DevComment": "Negative status effects"
    }
  ]
}
```

Load it in your app:

```rust
use bevy_gameplay_tag::{GameplayTagsPlugin, GameplayTagsSettings};

App::new()
    .add_plugins(GameplayTagsPlugin::new("assets/tag_data.json"))
    .run();
```

### 3. Use Tags in Your Game

```rust
use bevy::prelude::*;
use bevy_gameplay_tag::*;

fn setup(mut commands: Commands) {
    // Spawn an entity with a tag count container
    commands.spawn(GameplayTagCountContainer::new());
}

fn add_tags_system(
    mut query: Query<(Entity, &mut GameplayTagCountContainer)>,
    tags_manager: Res<GameplayTagsManager>,
    mut commands: Commands,
) {
    for (entity, mut tag_container) in query.iter_mut() {
        let fire_skill = GameplayTag::new("Ability.Skill.Fire");

        // Add a tag (increments count by 1)
        tag_container.update_tag_count(
            &fire_skill,
            1,
            &tags_manager,
            &mut commands,
            entity,
        );

        // Check if entity has the tag
        if tag_container.has_matching_gameplay_tag(&fire_skill) {
            println!("Entity has fire skill!");
        }

        // Check parent tags (hierarchical matching)
        let ability_tag = GameplayTag::new("Ability");
        if tag_container.has_matching_gameplay_tag(&ability_tag) {
            println!("Entity has some ability!");
        }
    }
}
```

## Core Concepts

### GameplayTag

The fundamental building block representing a single tag:

```rust
let tag = GameplayTag::new("Ability.Skill.Fire");

// Exact matching
tag.matches_tag_exact(&other_tag);

// Hierarchical matching (Fire matches Ability.Skill)
tag.matches_tag(&parent_tag, &tags_manager);
```

### GameplayTagContainer

A collection of tags with query capabilities:

```rust
let mut container = GameplayTagContainer::new();

// Add tags
container.add_tag(fire_tag, &tags_manager);
container.add_tag(ice_tag, &tags_manager);

// Query tags
container.has_tag(&fire_tag);              // Check for tag or parent
container.has_tag_exact(&fire_tag);        // Exact match only
container.has_any(&other_container);       // Any intersection
container.has_all(&required_tags);         // All tags present
```

### GameplayTagCountContainer

Reference-counted tags with event notifications:

```rust
let mut tag_container = GameplayTagCountContainer::new();

// Increment tag count
tag_container.update_tag_count(&tag, 1, &tags_manager, &mut commands, entity);

// Decrement tag count
tag_container.update_tag_count(&tag, -1, &tags_manager, &mut commands, entity);

// Set absolute count
tag_container.set_tag_count(&tag, 5, &tags_manager, &mut commands, entity);

// Get current count
let count = tag_container.get_tag_count(&tag);
```

### Tag Change Events

React to tag changes using Bevy's observer pattern:

```rust
fn setup(mut commands: Commands) {
    let entity = commands.spawn(GameplayTagCountContainer::new()).id();

    // Observe tag changes
    commands.entity(entity).observe(on_tag_changed);
}

fn on_tag_changed(trigger: Trigger<OnGameplayEffectTagCountChanged>) {
    let event = trigger.event();

    match event.event_type {
        GameplayTagEventType::NewOrRemoved => {
            println!("Tag {} was added or removed", event.tag);
        }
        GameplayTagEventType::AnyCountChanged => {
            println!("Tag {} count changed to {}", event.tag, event.tag_count);
        }
    }
}
```

### Complex Queries

Build sophisticated tag queries with boolean logic:

```rust
// Create a query expression
let mut expr = GameplayTagQueryExpression::new();
expr.all_tags_match()
    .add_tag(GameplayTag::new("Ability.Skill.Fire"));
expr.no_tags_match()
    .add_tag(GameplayTag::new("Status.Debuff.Silence"));

let query = GameplayTagQuery::new(expr);

// Test against a container
if query.matches(&container) {
    println!("Entity can cast fire skill!");
}
```

### Tag Requirements

Define declarative tag requirements:

```rust
let mut requirements = GameplayTagRequirements::new();

// Must have these tags
requirements.require_tags.add_tag(
    GameplayTag::new("Ability.Skill"),
    &tags_manager
);

// Must NOT have these tags
requirements.ignore_tags.add_tag(
    GameplayTag::new("Status.Debuff.Silence"),
    &tags_manager
);

// Check if requirements are met
if requirements.requirements_met(&entity_tags) {
    println!("Can use ability!");
}
```

## Use Cases

### Skill System

```rust
// Define skill tags
let fire_skill = GameplayTag::new("Ability.Skill.Fire");
let cooldown = GameplayTag::new("Cooldown.Skill.Fire");

// Cast skill
tag_container.update_tag_count(&fire_skill, 1, &tags_manager, &mut commands, entity);
tag_container.update_tag_count(&cooldown, 1, &tags_manager, &mut commands, entity);

// Check if skill is on cooldown
if tag_container.has_matching_gameplay_tag(&cooldown) {
    println!("Skill is on cooldown!");
}
```

### Buff/Debuff System

```rust
// Stack buffs with reference counting
let strength_buff = GameplayTag::new("Status.Buff.Strength");

// Add 3 stacks
tag_container.update_tag_count(&strength_buff, 3, &tags_manager, &mut commands, entity);

// Get stack count
let stacks = tag_container.get_tag_count(&strength_buff);
println!("Strength buff has {} stacks", stacks);
```

### State Machine

```rust
// Define states as tags
let idle = GameplayTag::new("State.Idle");
let running = GameplayTag::new("State.Running");
let jumping = GameplayTag::new("State.Jumping");

// Transition states
tag_container.set_tag_count(&idle, 0, &tags_manager, &mut commands, entity);
tag_container.set_tag_count(&running, 1, &tags_manager, &mut commands, entity);
```

### Team/Faction System

```rust
let player_team = GameplayTag::new("Teams.Player");
let monster_team = GameplayTag::new("Teams.Monster");

// Check if entities are on the same team
if entity1_tags.has_any(&entity2_tags) {
    println!("Same team!");
}
```

## Performance

- **String Interning**: Uses `string_cache` for efficient string storage and comparison
- **Binary Search**: O(log n) tag lookups in sorted containers
- **Lazy Updates**: Parent tags are updated only when necessary
- **Efficient Counting**: HashMap-based reference counting

## Examples

Check out the [examples](examples/) directory for complete working examples:

```bash
cargo run --example example
```

## Architecture

```
src/
├── lib.rs                          # Module exports
├── gameplay_tag.rs                 # Core tag definition
├── gameplay_tags_manager.rs        # Tag manager
├── gameplay_tag_container.rs       # Tag container and query system
├── gameplay_tag_count_container.rs # Reference-counted tag container
├── gameplay_tag_requirements.rs    # Tag requirements system
└── gameplay_tags_plugin.rs         # Bevy plugin integration
```

## Compatibility

| Bevy Version | Plugin Version |
|--------------|----------------|
| 0.18         | 0.1.0          |

## License

Licensed under either of:

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

## Acknowledgments

This project is inspired by Unreal Engine's Gameplay Tag system, adapted for the Rust and Bevy ecosystem.
