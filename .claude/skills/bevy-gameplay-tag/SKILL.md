---
name: bevy-gameplay-tag
description: Guide for using the bevy_gameplay_tag library - a hierarchical gameplay tag system for Bevy game engine inspired by Unreal Engine's Gameplay Tags. Use when working with bevy_gameplay_tag library, implementing tag-based game systems (skill systems, buff/debuff management, state machines, AI behavior conditions, item categorization), or when users ask about gameplay tags, hierarchical tag matching, tag containers, tag counting, or tag-based event systems in Bevy.
---

# Bevy Gameplay Tag

## Overview

`bevy_gameplay_tag` is a hierarchical gameplay tag system for Bevy, inspired by Unreal Engine's Gameplay Tags. It provides efficient tag management with parent-child relationships (e.g., `Ability.Skill.S1` where `Ability` is parent of `Ability.Skill`).

**Key capabilities:**
- Hierarchical tag matching (child tags match parent queries)
- Tag containers with efficient querying (O(log n) binary search)
- Reference counting for stacking effects (buffs/debuffs)
- Event-driven tag changes via Bevy observers
- Complex query expressions (any/all/none logic)
- JSON-based tag configuration

## Quick Start

### 1. Add Plugin to Bevy App

```rust
use bevy::prelude::*;
use bevy_gameplay_tag::gameplay_tags_plugin::GameplayTagsPlugin;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        // Option A: No pre-loaded tags
        .add_plugins(GameplayTagsPlugin::new())
        // Option B: Load tags from JSON file
        .add_plugins(GameplayTagsPlugin::with_data_path("assets/tags.json".to_string()))
        .run();
}
```

### 2. Create Tags JSON (Optional)

If using `with_data_path`, create a JSON file with tag definitions. See `assets/tags_template.json` in this skill for a starter template.

```json
[
  {
    "tag_name": "Ability.Skill.Attack",
    "description": "Attack skill",
    "path": ""
  },
  {
    "tag_name": "Status.Stunned",
    "description": "Stunned status effect",
    "path": ""
  }
]
```

### 3. Add Tags to Entities

```rust
use bevy_gameplay_tag::gameplay_tag_count_container::GameplayTagCountContainer;

fn spawn_player(mut commands: Commands) {
    commands.spawn((
        Name::new("Player"),
        GameplayTagCountContainer::new(),
    ));
}
```

## Common Use Cases

### Use Case 1: Skill System with Cooldowns

```rust
use bevy_gameplay_tag::{
    gameplay_tag::GameplayTag,
    gameplay_tag_count_container::GameplayTagCountContainer,
    gameplay_tags_manager::GameplayTagsManager,
};

fn use_skill_system(
    mut query: Query<(Entity, &mut GameplayTagCountContainer)>,
    tags_manager: Res<GameplayTagsManager>,
    mut commands: Commands,
    keyboard: Res<ButtonInput<KeyCode>>,
) {
    if keyboard.just_pressed(KeyCode::Space) {
        for (entity, mut tags) in query.iter_mut() {
            let cooldown_tag = GameplayTag::new("Cooldown.Skill.Attack");

            // Check if skill is on cooldown
            if !tags.has_matching_gameplay_tag(&cooldown_tag) {
                // Use skill
                info!("Skill used!");

                // Add cooldown tag
                tags.update_tag_count(
                    &cooldown_tag,
                    1,
                    &tags_manager,
                    &mut commands,
                    entity,
                );
            } else {
                info!("Skill on cooldown!");
            }
        }
    }
}
```

### Use Case 2: Buff/Debuff System with Stacking

```rust
fn apply_buff_system(
    mut query: Query<(Entity, &mut GameplayTagCountContainer)>,
    tags_manager: Res<GameplayTagsManager>,
    mut commands: Commands,
) {
    for (entity, mut tags) in query.iter_mut() {
        let buff_tag = GameplayTag::new("Buff.Strength");

        // Add 3 stacks of strength buff
        tags.set_tag_count(&buff_tag, 3, &tags_manager, &mut commands, entity);

        // Or increment by 1
        tags.update_tag_count(&buff_tag, 1, &tags_manager, &mut commands, entity);

        // Check current stacks
        let stacks = tags.get_tag_count(&buff_tag);
        info!("Strength buff stacks: {}", stacks);
    }
}
```

### Use Case 3: Condition Checking with Hierarchical Matching

```rust
use bevy_gameplay_tag::gameplay_tag_container::GameplayTagContainer;

fn can_perform_action(
    entity_tags: &GameplayTagContainer,
) -> bool {
    let stunned = GameplayTag::new("Status.Stunned");
    let frozen = GameplayTag::new("Status.Frozen");

    // Can act if not stunned or frozen
    !entity_tags.has_tag(&stunned) && !entity_tags.has_tag(&frozen)
}

fn check_skill_type(entity_tags: &GameplayTagContainer) -> bool {
    // Check if entity has any skill tag
    // This matches "Ability.Skill.Attack", "Ability.Skill.Defend", etc.
    let skill_parent = GameplayTag::new("Ability.Skill");
    entity_tags.has_tag(&skill_parent)  // Hierarchical match
}
```

### Use Case 4: Event-Driven Tag Changes

```rust
use bevy_gameplay_tag::gameplay_tag_count_container::{
    OnGameplayEffectTagCountChanged,
    GameplayTagEventType,
};

fn setup(mut commands: Commands) {
    let entity = commands
        .spawn((
            Name::new("Player"),
            GameplayTagCountContainer::new(),
        ))
        .id();

    // Observe tag changes on this entity
    commands.entity(entity).observe(on_tag_changed);
}

fn on_tag_changed(
    trigger: On<OnGameplayEffectTagCountChanged>,
    query: Query<&Name>,
) {
    let event = trigger.event();
    let name = query.get(event.entity).unwrap();

    match event.event_type {
        GameplayTagEventType::NewOrRemoved => {
            if event.new_count > 0 {
                info!("{} gained tag: {:?}", name, event.tag);
            } else {
                info!("{} lost tag: {:?}", name, event.tag);
            }
        }
        GameplayTagEventType::AnyCountChanged => {
            info!("{} tag {:?} count: {}", name, event.tag, event.new_count);
        }
    }
}
```

### Use Case 5: Complex Query Expressions

```rust
use bevy_gameplay_tag::gameplay_tag_container::GameplayTagQueryExpression;

fn check_complex_conditions(entity_tags: &GameplayTagContainer) -> bool {
    // Can use skill if:
    // - Has ANY active skill tag (Attack OR Defend)
    // - NOT stunned or frozen
    let has_skill = GameplayTagQueryExpression::any_tags_match(vec![
        GameplayTag::new("Ability.Skill.Attack"),
        GameplayTag::new("Ability.Skill.Defend"),
    ]);

    let not_disabled = GameplayTagQueryExpression::no_tags_match(vec![
        GameplayTag::new("Status.Stunned"),
        GameplayTag::new("Status.Frozen"),
    ]);

    entity_tags.matches_query(&has_skill) && entity_tags.matches_query(&not_disabled)
}
```

### Use Case 6: Item Filtering by Tags

```rust
fn find_weapons(
    items: Query<(Entity, &GameplayTagContainer)>,
) -> Vec<Entity> {
    let weapon_tag = GameplayTag::new("Item.Type.Weapon");

    items
        .iter()
        .filter(|(_, tags)| tags.has_tag(&weapon_tag))
        .map(|(entity, _)| entity)
        .collect()
}

fn find_equipment(
    items: Query<(Entity, &GameplayTagContainer)>,
) -> Vec<Entity> {
    // Matches "Item.Type.Equipment.Weapon", "Item.Type.Equipment.Armor", etc.
    let equipment_tag = GameplayTag::new("Item.Type.Equipment");

    items
        .iter()
        .filter(|(_, tags)| tags.has_tag(&equipment_tag))
        .map(|(entity, _)| entity)
        .collect()
}
```

## Key Concepts

### Hierarchical Matching

Tags use dot notation for hierarchy: `Parent.Child.Grandchild`

```rust
// Entity has "Ability.Skill.Attack"
let entity_tags = /* ... */;

// Hierarchical matching (has_tag)
entity_tags.has_tag(&GameplayTag::new("Ability.Skill.Attack")); // true
entity_tags.has_tag(&GameplayTag::new("Ability.Skill"));        // true
entity_tags.has_tag(&GameplayTag::new("Ability"));              // true

// Exact matching (has_tag_exact)
entity_tags.has_tag_exact(&GameplayTag::new("Ability"));        // false
entity_tags.has_tag_exact(&GameplayTag::new("Ability.Skill.Attack")); // true
```

### Container Types

**GameplayTagContainer**: Basic container for tag sets
- Use for: Static tag collections, item properties, entity types
- Methods: `add_tag`, `remove_tag`, `has_tag`, `has_any`, `has_all`

**GameplayTagCountContainer**: Container with reference counting
- Use for: Stacking effects, buffs/debuffs, temporary states
- Methods: `update_tag_count`, `set_tag_count`, `get_tag_count`
- Triggers events when counts change

### Manager Access

`GameplayTagsManager` is available as a Bevy resource after adding the plugin:

```rust
fn system(tags_manager: Res<GameplayTagsManager>) {
    // Use manager for tag operations
}
```

Always pass `&tags_manager` when adding/removing tags to maintain hierarchy.

## Tag Naming Conventions

Recommended hierarchical structure:

```
Ability.Skill.<SkillName>      - Active skills
Ability.Passive.<PassiveName>  - Passive abilities
Status.<StatusName>            - Status effects
Cooldown.Skill.<SkillName>     - Skill cooldowns
Buff.<BuffName>                - Positive effects
Debuff.<DebuffName>            - Negative effects
Item.Type.<Category>           - Item categories
Item.Rarity.<Rarity>           - Item rarity
Teams.<TeamName>               - Team/faction tags
```

## Common Patterns

### Pattern: Ability Requirements

```rust
use bevy_gameplay_tag::gameplay_tag_requirements::GameplayTagRequirements;

fn check_ability_requirements(
    entity_tags: &GameplayTagContainer,
    requirements: &GameplayTagRequirements,
) -> bool {
    requirements.requires_met(entity_tags)
}

// Define requirements
let mut require_tags = GameplayTagContainer::new();
require_tags.add_tag(GameplayTag::new("Status.Alive"), &tags_manager);

let mut ignore_tags = GameplayTagContainer::new();
ignore_tags.add_tag(GameplayTag::new("Status.Stunned"), &tags_manager);

let requirements = GameplayTagRequirements {
    require_tags,
    ignore_tags,
    tag_query: None,
};
```

### Pattern: Tag-Based State Machine

```rust
fn update_state(
    mut query: Query<(Entity, &mut GameplayTagCountContainer)>,
    tags_manager: Res<GameplayTagsManager>,
    mut commands: Commands,
) {
    for (entity, mut tags) in query.iter_mut() {
        if tags.has_matching_gameplay_tag(&GameplayTag::new("State.Idle")) {
            // Transition to moving
            tags.set_tag_count(
                &GameplayTag::new("State.Idle"),
                0,
                &tags_manager,
                &mut commands,
                entity,
            );
            tags.set_tag_count(
                &GameplayTag::new("State.Moving"),
                1,
                &tags_manager,
                &mut commands,
                entity,
            );
        }
    }
}
```

## Detailed API Reference

For comprehensive API documentation including all methods, parameters, and advanced patterns, see [references/api_reference.md](references/api_reference.md).

## Assets

- `assets/tags_template.json` - Starter template for tag definitions. Copy and customize for your game.

## Best Practices

1. **Use hierarchical tags** - Design tag hierarchies that reflect your game's structure
2. **Prefer GameplayTagCountContainer** - Use for entities that need dynamic tag changes
3. **Use observers for reactions** - Attach observers to entities that need to react to tag changes
4. **Cache tag instances** - Create `GameplayTag` instances once and reuse them
5. **Use exact matching sparingly** - Hierarchical matching is more flexible
6. **Define tags in JSON** - Centralize tag definitions for easier management
7. **Namespace your tags** - Use clear prefixes (Ability, Status, Item, etc.)
