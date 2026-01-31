---
name: bevy-gameplay-tag
description: Guide for using the bevy_gameplay_tag library - a hierarchical gameplay tag system for Bevy game engine inspired by Unreal Engine's Gameplay Tags. Use when working with bevy_gameplay_tag library, implementing tag-based game systems (skill systems, buff/debuff management, state machines, AI behavior conditions, item categorization), or when users ask about gameplay tags, hierarchical tag matching, tag containers, tag counting, or tag-based event systems in Bevy.
---

# Bevy Gameplay Tag

## Overview

`bevy_gameplay_tag` is a hierarchical gameplay tag system for Bevy, inspired by Unreal Engine's Gameplay Tags. It provides efficient tag management with parent-child relationships (e.g., `Ability.Skill.Fire` where `Ability` is parent of `Ability.Skill`).

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
use bevy_gameplay_tag::GameplayTagsPlugin;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        // Option A: Use default (inline JSON configuration)
        .add_plugins(GameplayTagsPlugin::default())
        // Option B: Load tags from JSON file
        .add_plugins(GameplayTagsPlugin::new("assets/tag_data.json"))
        .run();
}
```

### 2. Create Tags JSON (Optional)

If using external JSON file, create a `tag_data.json` file. See `assets/tags_template.json` in this skill for a starter template.

```json
{
  "GameplayTagList": [
    {
      "Tag": "Ability.Skill.Fire",
      "DevComment": "Fire skill"
    },
    {
      "Tag": "Status.Buff.Strength",
      "DevComment": "Strength buff"
    },
    {
      "Tag": "Cooldown.Skill.Fire",
      "DevComment": "Fire skill cooldown"
    }
  ]
}
```

### 3. Add Tags to Entities

```rust
use bevy_gameplay_tag::GameplayTagCountContainer;

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
use bevy_gameplay_tag::{GameplayTag, GameplayTagCountContainer, GameplayTagsManager};

fn use_skill_system(
    mut query: Query<(Entity, &mut GameplayTagCountContainer)>,
    tags_manager: Res<GameplayTagsManager>,
    mut commands: Commands,
    keyboard: Res<ButtonInput<KeyCode>>,
) {
    if keyboard.just_pressed(KeyCode::Space) {
        for (entity, mut tags) in query.iter_mut() {
            let cooldown_tag = GameplayTag::new("Cooldown.Skill.Fire");

            // Check if skill is on cooldown
            if !tags.has_matching_gameplay_tag(&cooldown_tag) {
                // Use skill
                info!("Fire skill used!");

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
        let buff_tag = GameplayTag::new("Status.Buff.Strength");

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
use bevy_gameplay_tag::GameplayTagContainer;

fn can_perform_action(entity_tags: &GameplayTagContainer) -> bool {
    let stunned = GameplayTag::new("Status.Debuff.Stunned");
    let frozen = GameplayTag::new("Status.Debuff.Frozen");

    // Can act if not stunned or frozen
    !entity_tags.has_tag(&stunned) && !entity_tags.has_tag(&frozen)
}

fn check_skill_type(entity_tags: &GameplayTagContainer) -> bool {
    // Check if entity has any skill tag
    // This matches "Ability.Skill.Fire", "Ability.Skill.Ice", etc.
    let skill_parent = GameplayTag::new("Ability.Skill");
    entity_tags.has_tag(&skill_parent)  // Hierarchical match
}
```

### Use Case 4: Event-Driven Tag Changes

```rust
use bevy_gameplay_tag::{OnGameplayEffectTagCountChanged, GameplayTagEventType};

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
    trigger: Trigger<OnGameplayEffectTagCountChanged>,
    query: Query<&Name>,
) {
    let event = trigger.event();
    let name = query.get(trigger.entity()).unwrap();

    match event.event_type {
        GameplayTagEventType::NewOrRemoved => {
            if event.tag_count > 0 {
                info!("{} gained tag: {:?}", name, event.tag);
            } else {
                info!("{} lost tag: {:?}", name, event.tag);
            }
        }
        GameplayTagEventType::AnyCountChanged => {
            info!("{} tag {:?} count: {}", name, event.tag, event.tag_count);
        }
    }
}
```

### Use Case 5: Complex Query Expressions

```rust
use bevy_gameplay_tag::{GameplayTagQuery, GameplayTagQueryExpression};

fn check_complex_conditions(entity_tags: &GameplayTagContainer) -> bool {
    // Can use skill if:
    // - Has ANY active skill tag (Fire OR Ice)
    // - NOT stunned or frozen
    let mut expr = GameplayTagQueryExpression::new();

    expr.any_tags_match()
        .add_tag(GameplayTag::new("Ability.Skill.Fire"))
        .add_tag(GameplayTag::new("Ability.Skill.Ice"));

    expr.no_tags_match()
        .add_tag(GameplayTag::new("Status.Debuff.Stunned"))
        .add_tag(GameplayTag::new("Status.Debuff.Frozen"));

    let query = GameplayTagQuery::new(expr);
    query.matches(entity_tags)
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

### Use Case 7: Team/Faction System

```rust
fn check_same_team(
    entity1_tags: &GameplayTagContainer,
    entity2_tags: &GameplayTagContainer,
) -> bool {
    // Check if entities share any team tag
    entity1_tags.has_any(entity2_tags)
}

fn is_enemy(
    player_tags: &GameplayTagContainer,
    target_tags: &GameplayTagContainer,
) -> bool {
    let player_team = GameplayTag::new("Teams.Player");
    let monster_team = GameplayTag::new("Teams.Monster");

    player_tags.has_tag(&player_team) && target_tags.has_tag(&monster_team)
}
```

## Key Concepts

### Hierarchical Matching

Tags use dot notation for hierarchy: `Parent.Child.Grandchild`

```rust
// Entity has "Ability.Skill.Fire"
let entity_tags = /* ... */;

// Hierarchical matching (has_tag)
entity_tags.has_tag(&GameplayTag::new("Ability.Skill.Fire")); // true
entity_tags.has_tag(&GameplayTag::new("Ability.Skill"));      // true
entity_tags.has_tag(&GameplayTag::new("Ability"));            // true

// Exact matching (has_tag_exact)
entity_tags.has_tag_exact(&GameplayTag::new("Ability"));      // false
entity_tags.has_tag_exact(&GameplayTag::new("Ability.Skill.Fire")); // true
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
Status.Buff.<BuffName>         - Positive effects
Status.Debuff.<DebuffName>     - Negative effects
Cooldown.Skill.<SkillName>     - Skill cooldowns
Item.Type.<Category>           - Item categories
Item.Rarity.<Rarity>           - Item rarity
Teams.<TeamName>               - Team/faction tags
State.<StateName>              - Entity states
AI.State.<StateName>           - AI states
AI.Perception.<Type>           - AI perception
```

## Common Patterns

### Pattern: Ability Requirements

```rust
use bevy_gameplay_tag::GameplayTagRequirements;

fn check_ability_requirements(
    entity_tags: &GameplayTagContainer,
    requirements: &GameplayTagRequirements,
) -> bool {
    requirements.requirements_met(entity_tags)
}

// Define requirements
let mut requirements = GameplayTagRequirements::new();
requirements.require_tags.add_tag(
    GameplayTag::new("Status.Alive"),
    &tags_manager
);
requirements.ignore_tags.add_tag(
    GameplayTag::new("Status.Debuff.Stunned"),
    &tags_manager
);
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

### Pattern: Cooldown Timer

```rust
fn cooldown_timer_system(
    mut query: Query<(Entity, &mut GameplayTagCountContainer)>,
    tags_manager: Res<GameplayTagsManager>,
    mut commands: Commands,
    time: Res<Time>,
) {
    for (entity, mut tags) in query.iter_mut() {
        let cooldown_tag = GameplayTag::new("Cooldown.Skill.Fire");

        if tags.has_matching_gameplay_tag(&cooldown_tag) {
            // Remove cooldown after some time
            // In real implementation, you'd track time separately
            tags.update_tag_count(
                &cooldown_tag,
                -1,
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
8. **Keep hierarchies shallow** - Avoid deeply nested tags (3-4 levels max)
9. **Use consistent naming** - Follow a naming convention across your project
10. **Document your tags** - Use DevComment field in JSON to explain tag purposes
