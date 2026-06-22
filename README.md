# Bevy Gameplay Tag

A hierarchical gameplay tag system for the Bevy game engine, inspired by Unreal Engine's Gameplay Tags.

[![License](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)
[![Bevy](https://img.shields.io/badge/Bevy-0.19-blue)](https://bevyengine.org)

[English](README.md) | [简体中文](README_zh.md)

## What this crate provides

`bevy_gameplay_tag` gives you a shared vocabulary for gameplay state such as abilities, cooldowns, buffs, debuffs, factions, AI state, and item categories.

It focuses on four core capabilities:

- **Hierarchical tags**: `Ability.Skill.Fire` also matches `Ability.Skill` and `Ability`
- **Container queries**: test whether an entity has any/all matching tags
- **Reference-counted tags**: support stacked effects and multiple sources contributing the same tag
- **Declarative requirements and queries**: express allow/block rules without scattering string checks through gameplay code

## Quick start

Add the crate to your `Cargo.toml`:

```toml
[dependencies]
bevy_gameplay_tag = "0.3.0"
bevy = "0.19.0"
```

Load the plugin and spawn a tag container:

```rust
use bevy::prelude::*;
use bevy_gameplay_tag::{GameplayTagCountContainer, GameplayTagsPlugin};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(GameplayTagsPlugin::with_data_path(
            "assets/tag_data.json",
        ))
        .add_systems(Startup, setup)
        .run();
}

fn setup(mut commands: Commands) {
    commands.spawn(GameplayTagCountContainer::new());
}
```

## Tag configuration format

The current loader expects a top-level JSON array. Each row contains:

- `tag_name`: the full hierarchical tag name
- `description`: free-form description text

Example `tag_data.json`:

```json
[
  {
    "tag_name": "Ability.Skill.Fire",
    "description": "Fire skill"
  },
  {
    "tag_name": "Status.Buff.Haste",
    "description": "Movement speed increase"
  },
  {
    "tag_name": "Status.Debuff.Silence",
    "description": "Cannot cast abilities"
  }
]
```

If you want to validate JSON before building the Bevy app, use:

```rust
use bevy_gameplay_tag::GameplayTagsSettings;

let rows = GameplayTagsSettings::parse_tag_table(json_source)?;
println!("Loaded {} tag rows", rows.len());
```

## API map

### Primary entry points

- `GameplayTag` — one immutable tag value
- `GameplayTagContainer` — explicit tags plus inherited parent-tag matching
- `GameplayTagCountContainer` — counted/stacked tag presence for entities
- `GameplayTagRequirements` — required + blocked + query-based conditions
- `GameplayTagsManager` — resource containing the loaded hierarchy
- `GameplayTagsPlugin` — plugin that initializes the manager

### Advanced entry points

- `GameplayTagQuery` — prebuilt query object
- `GameplayTagQueryExpression` — boolean expression builder for advanced filtering
- `OnGameplayEffectTagCountChanged` — observer event fired on count changes
- `GameplayTagEventType` — event kind (`NewOrRemoved` / `AnyCountChanged`)

## Choosing the right container

### `GameplayTagContainer`

Use this when you want a set-like container of explicit tags and hierarchical matching.

Typical use cases:

- ability categories
- faction or team labels
- static item classification
- simple state tags with no stacking

```rust
use bevy_gameplay_tag::{GameplayTag, GameplayTagContainer};

let mut tags = GameplayTagContainer::new();
tags.add_tag(GameplayTag::new("Ability.Skill.Fire"), &tags_manager);

assert!(tags.has_tag(&GameplayTag::new("Ability")));
assert!(tags.has_tag(&GameplayTag::new("Ability.Skill")));
assert!(tags.has_tag_exact(&GameplayTag::new("Ability.Skill.Fire")));
```

### `GameplayTagCountContainer`

Use this when the same tag can be contributed by multiple sources or needs stack counts.

Typical use cases:

- buff/debuff stacks
- cooldown sources
- temporary blocked states from several systems
- layered gameplay effects

```rust
use bevy_gameplay_tag::GameplayTag;

let tag = GameplayTag::new("Status.Buff.Haste");
tag_container.update_tag_count(&tag, 3, &tags_manager, &mut commands, entity);

assert_eq!(tag_container.get_tag_count(&tag), 3);
assert!(tag_container.has_tag(&tag));
```

## Common tasks

### Add a tag and test parent matching

```rust
use bevy_gameplay_tag::{GameplayTag, GameplayTagContainer};

let fire = GameplayTag::new("Ability.Skill.Fire");
let mut tags = GameplayTagContainer::new();
tags.add_tag(fire.clone(), &tags_manager);

assert!(tags.has_tag(&GameplayTag::new("Ability")));
assert!(tags.has_tag(&GameplayTag::new("Ability.Skill")));
assert!(tags.has_tag_exact(&fire));
```

### Counted tags for stacked effects

```rust
use bevy_gameplay_tag::GameplayTag;

let buff = GameplayTag::new("Status.Buff.Haste");
tag_container.update_tag_count(&buff, 1, &tags_manager, &mut commands, entity);
tag_container.update_tag_count(&buff, 1, &tags_manager, &mut commands, entity);

assert_eq!(tag_container.get_explicit_tag_count(&buff), 2);
```

### Listen for tag count changes

```rust
use bevy::prelude::*;
use bevy_gameplay_tag::{
    GameplayTagCountContainer, GameplayTagEventType, OnGameplayEffectTagCountChanged,
};

fn setup(mut commands: Commands) {
    let entity = commands.spawn(GameplayTagCountContainer::new()).id();
    commands.entity(entity).observe(on_tag_changed);
}

fn on_tag_changed(trigger: On<OnGameplayEffectTagCountChanged>) {
    let event = trigger.event();

    match event.event_type {
        GameplayTagEventType::NewOrRemoved => {
            println!("tag {:?} entered or left the active set", event.tag);
        }
        GameplayTagEventType::AnyCountChanged => {
            println!("tag {:?} count is now {}", event.tag, event.new_count);
        }
    }
}
```

### Declarative requirements

```rust
use bevy_gameplay_tag::{GameplayTag, GameplayTagQuery, GameplayTagRequirements};

let mut requirements = GameplayTagRequirements::new();
requirements
    .require_tags_mut()
    .add_tag(GameplayTag::new("Ability.Skill.Fire"), &tags_manager);
requirements
    .ignore_tags_mut()
    .add_tag(GameplayTag::new("Status.Debuff.Silence"), &tags_manager);

if requirements.matches(&entity_tags) {
    println!("Entity can use the fire skill");
}

let query: GameplayTagQuery = requirements.to_query();
```

## Advanced queries

For simple checks, `has_tag`, `has_any`, `has_all`, and `GameplayTagRequirements` are usually enough.

Use `GameplayTagQueryExpression` when you need nested boolean logic:

```rust
use bevy_gameplay_tag::{GameplayTag, GameplayTagQuery, GameplayTagQueryExpression};

let mut required = GameplayTagQueryExpression::new();
required
    .all_tags_match()
    .add_tag(GameplayTag::new("Ability.Skill.Fire"));

let mut blocked = GameplayTagQueryExpression::new();
blocked
    .no_tags_match()
    .add_tag(GameplayTag::new("Status.Debuff.Silence"));

let mut root = GameplayTagQueryExpression::new();
root.all_expr_match().add_expr(required).add_expr(blocked);

let mut query = GameplayTagQuery::new();
query.build(root);

if query.matches(&entity_tags) {
    println!("Entity passes the advanced tag query");
}
```

For convenience, `GameplayTagQuery` also provides:

- `GameplayTagQuery::match_any(&container)`
- `GameplayTagQuery::match_all(&container)`
- `GameplayTagQuery::match_none(&container)`

## Error handling and current limitations

- `GameplayTag::try_new(...)` validates tag names up front and rejects empty names, leading/trailing dots, repeated separators, and non-`[A-Za-z0-9_]` segments.
- `GameplayTagsSettings::parse_tag_table(...)` and `GameplayTagsSettings::load_tag_table_from_path(...)` validate all rows and return explicit errors for invalid JSON, invalid tag names, and duplicate tag definitions.
- `GameplayTagsPlugin` still uses log-based initialization. If file loading or JSON parsing fails during plugin setup, the crate logs the error and falls back to an empty tag table.
- If you need explicit failure handling, use `GameplayTagsSettings::parse_tag_table(...)` or `GameplayTagsSettings::load_tag_table_from_path(...)` before starting your app.
- The crate currently uses runtime tag strings rather than generated constants or compile-time validation.
- Some rustdoc examples are intentionally marked `ignore` because they require a populated `GameplayTagsManager` context.

## Example app

Run the included example:

```bash
cargo run --example example
```

The example demonstrates:

- loading tags from `examples/tag_data.json`
- attaching `GameplayTagCountContainer` to entities
- observing tag change events
- checking hierarchical matches at runtime

## Architecture

```text
src/
├── lib.rs                          # Module exports
├── gameplay_tag.rs                 # Core tag definition
├── gameplay_tags_manager.rs        # Tag loading and hierarchy management
├── gameplay_tag_container.rs       # Set-style tag container and query expressions
├── gameplay_tag_count_container.rs # Reference-counted tag container and events
├── gameplay_tag_requirements.rs    # Declarative requirements wrapper
└── gameplay_tags_plugin.rs         # Bevy plugin integration
```

## Compatibility

| Bevy Version | Plugin Version |
|--------------|----------------|
| 0.19.0       | 0.3.0          |

## License

Licensed under the MIT license ([LICENSE](LICENSE) or http://opensource.org/licenses/MIT).

## Acknowledgments

This project is inspired by Unreal Engine's Gameplay Tag system, adapted for the Rust and Bevy ecosystem.
