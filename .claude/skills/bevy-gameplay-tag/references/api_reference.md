# bevy_gameplay_tag API Reference

## Core Types

### GameplayTag

Represents a single gameplay tag with hierarchical structure (e.g., `Ability.Skill.S1`).

```rust
use bevy_gameplay_tag::gameplay_tag::GameplayTag;

// Create a tag
let tag = GameplayTag::new("Ability.Skill.S1");

// Tags support hierarchy: "A.B.C" where A is parent of A.B, and A.B is parent of A.B.C
```

**Key Methods:**
- `new(name: &str) -> Self` - Create a new tag
- `matches_tag(&self, other: &GameplayTag) -> bool` - Check if tags match (supports hierarchy)
- `matches_tag_exact(&self, other: &GameplayTag) -> bool` - Exact match only

### GameplayTagContainer

Container for multiple tags with efficient querying.

```rust
use bevy_gameplay_tag::gameplay_tag_container::GameplayTagContainer;

let mut container = GameplayTagContainer::new();
```

**Key Methods:**

**Adding/Removing:**
- `add_tag(&mut self, tag: GameplayTag, manager: &GameplayTagsManager)`
- `add_tags(&mut self, tags: &GameplayTagContainer, manager: &GameplayTagsManager)`
- `remove_tag(&mut self, tag: &GameplayTag, defer_parent_removal: bool, manager: &GameplayTagsManager)`
- `remove_tags(&mut self, tags: &GameplayTagContainer, manager: &GameplayTagsManager)`

**Querying:**
- `has_tag(&self, tag: &GameplayTag) -> bool` - Hierarchical match
- `has_tag_exact(&self, tag: &GameplayTag) -> bool` - Exact match only
- `has_any(&self, other: &GameplayTagContainer) -> bool` - Any tag matches (hierarchical)
- `has_any_exact(&self, other: &GameplayTagContainer) -> bool` - Any exact match
- `has_all(&self, other: &GameplayTagContainer) -> bool` - All tags match (hierarchical)
- `has_all_exact(&self, other: &GameplayTagContainer) -> bool` - All exact matches

**Filtering:**
- `filter(&self, other: &GameplayTagContainer) -> GameplayTagContainer` - Keep only matching tags
- `matches_query(&self, query: &GameplayTagQueryExpression) -> bool` - Complex query matching

### GameplayTagCountContainer

Advanced container that maintains reference counts for each tag. Useful for stacking effects (buffs/debuffs).

```rust
use bevy_gameplay_tag::gameplay_tag_count_container::GameplayTagCountContainer;

let mut count_container = GameplayTagCountContainer::new();
```

**Key Methods:**
- `update_tag_count(&mut self, tag: &GameplayTag, delta: i32, manager: &GameplayTagsManager, commands: &mut Commands, entity: Entity)` - Add/subtract count
- `set_tag_count(&mut self, tag: &GameplayTag, count: i32, manager: &GameplayTagsManager, commands: &mut Commands, entity: Entity)` - Set absolute count
- `get_tag_count(&self, tag: &GameplayTag) -> i32` - Get current count
- `has_matching_gameplay_tag(&self, tag: &GameplayTag) -> bool` - Check if tag exists (count > 0)

**Events:**
When tag counts change, the system triggers `OnGameplayEffectTagCountChanged` events:
- `GameplayTagEventType::NewOrRemoved` - Tag first added or completely removed
- `GameplayTagEventType::AnyCountChanged` - Any count change

### GameplayTagsManager

Central manager for tag registration and hierarchy. Available as Bevy resource.

```rust
use bevy_gameplay_tag::gameplay_tags_manager::GameplayTagsManager;

fn system(tags_manager: Res<GameplayTagsManager>) {
    // Manager is automatically available after adding GameplayTagsPlugin
}
```

**Key Methods:**
- `request_gameplay_tag(&mut self, tag_name: &str) -> Option<GameplayTag>` - Get or create tag
- `add_tag_to_container(&self, tag: &GameplayTag, container: &mut GameplayTagContainer)` - Add tag with parents

### GameplayTagQueryExpression

Build complex logical queries for tag matching.

```rust
use bevy_gameplay_tag::gameplay_tag_container::GameplayTagQueryExpression;

// Match any of these tags
let query = GameplayTagQueryExpression::any_tags_match(vec![
    GameplayTag::new("Status.Stunned"),
    GameplayTag::new("Status.Frozen"),
]);

// Match all tags
let query = GameplayTagQueryExpression::all_tags_match(vec![
    GameplayTag::new("Ability.Active"),
    GameplayTag::new("Ability.Skill"),
]);

// No tags match
let query = GameplayTagQueryExpression::no_tags_match(vec![
    GameplayTag::new("Cooldown.Skill.S1"),
]);

// Check if container matches query
if container.matches_query(&query) {
    // ...
}
```

### GameplayTagRequirements

Declarative way to define tag requirements for abilities, items, etc.

```rust
use bevy_gameplay_tag::gameplay_tag_requirements::GameplayTagRequirements;

let requirements = GameplayTagRequirements {
    require_tags: some_container,  // Must have these tags
    ignore_tags: other_container,  // Must NOT have these tags
    tag_query: Some(query),        // Custom query expression
};

if requirements.requires_met(&entity_tags) {
    // Requirements satisfied
}
```

## Plugin Integration

### GameplayTagsPlugin

Add to Bevy app to enable the tag system.

```rust
use bevy_gameplay_tag::gameplay_tags_plugin::GameplayTagsPlugin;

// Default initialization (no pre-loaded tags)
app.add_plugins(GameplayTagsPlugin::new());

// Load tags from JSON file
app.add_plugins(GameplayTagsPlugin::with_data_path("assets/tags.json".to_string()));
```

### JSON Tag Data Format

```json
[
  {
    "tag_name": "Ability.Skill.S1",
    "description": "First skill",
    "path": ""
  },
  {
    "tag_name": "Status.Stunned",
    "description": "Stunned status effect",
    "path": ""
  }
]
```

## Common Patterns

### Pattern 1: Entity with Tags

```rust
fn spawn_entity(mut commands: Commands) {
    commands.spawn((
        Name::new("Player"),
        GameplayTagCountContainer::new(),
    ));
}
```

### Pattern 2: Observing Tag Changes

```rust
fn setup(mut commands: Commands) {
    let entity = commands
        .spawn(GameplayTagCountContainer::new())
        .id();

    commands.entity(entity).observe(on_tag_changed);
}

fn on_tag_changed(
    trigger: On<OnGameplayEffectTagCountChanged>,
    query: Query<&Name>
) {
    let event = trigger.event();

    match event.event_type {
        GameplayTagEventType::NewOrRemoved => {
            if event.new_count > 0 {
                info!("Tag added: {:?}", event.tag);
            } else {
                info!("Tag removed: {:?}", event.tag);
            }
        }
        GameplayTagEventType::AnyCountChanged => {
            info!("Count changed to: {}", event.new_count);
        }
    }
}
```

### Pattern 3: Checking Conditions

```rust
fn can_use_skill(
    entity_tags: &GameplayTagContainer,
    tags_manager: &GameplayTagsManager,
) -> bool {
    let cooldown_tag = GameplayTag::new("Cooldown.Skill.S1");
    let stunned_tag = GameplayTag::new("Status.Stunned");

    // Can use if not on cooldown and not stunned
    !entity_tags.has_tag(&cooldown_tag) && !entity_tags.has_tag(&stunned_tag)
}
```

### Pattern 4: Hierarchical Matching

```rust
// Entity has "Ability.Skill.S1"
let entity_tags = /* ... */;

// These all match due to hierarchy
entity_tags.has_tag(&GameplayTag::new("Ability.Skill.S1")); // true
entity_tags.has_tag(&GameplayTag::new("Ability.Skill"));    // true
entity_tags.has_tag(&GameplayTag::new("Ability"));          // true

// Exact match only
entity_tags.has_tag_exact(&GameplayTag::new("Ability"));    // false
entity_tags.has_tag_exact(&GameplayTag::new("Ability.Skill.S1")); // true
```
