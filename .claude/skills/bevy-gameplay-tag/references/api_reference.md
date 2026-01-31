# bevy_gameplay_tag API Reference

## Core Types

### GameplayTag

Represents a single gameplay tag with hierarchical structure (e.g., `Ability.Skill.Fire`).

```rust
use bevy_gameplay_tag::GameplayTag;

// Create a tag
let tag = GameplayTag::new("Ability.Skill.Fire");

// Tags support hierarchy: "A.B.C" where A is parent of A.B, and A.B is parent of A.B.C
```

**Key Methods:**
- `new(name: &str) -> Self` - Create a new tag from a string
- `matches_tag(&self, other: &GameplayTag, manager: &GameplayTagsManager) -> bool` - Check if tags match hierarchically
- `matches_tag_exact(&self, other: &GameplayTag) -> bool` - Exact match only (no hierarchy)
- `matches_any(&self, container: &GameplayTagContainer, manager: &GameplayTagsManager) -> bool` - Check if tag matches any in container
- `matches_any_exact(&self, container: &GameplayTagContainer) -> bool` - Exact match against any in container

### GameplayTagContainer

Container for multiple tags with efficient querying. Maintains both explicit tags and parent tags.

```rust
use bevy_gameplay_tag::GameplayTagContainer;

let mut container = GameplayTagContainer::new();
```

**Key Methods:**

**Adding/Removing:**
- `add_tag(&mut self, tag: GameplayTag, manager: &GameplayTagsManager)` - Add a tag and its parents
- `add_tags(&mut self, tags: &GameplayTagContainer)` - Add multiple tags
- `append_tags(&mut self, tags: &GameplayTagContainer)` - Append tags from another container
- `remove_tag(&mut self, tag: &GameplayTag)` - Remove a specific tag
- `remove_tags(&mut self, tags: &GameplayTagContainer)` - Remove multiple tags

**Querying:**
- `has_tag(&self, tag: &GameplayTag) -> bool` - Hierarchical match (checks explicit and parent tags)
- `has_tag_exact(&self, tag: &GameplayTag) -> bool` - Exact match only (checks explicit tags)
- `has_any(&self, other: &GameplayTagContainer) -> bool` - Any tag matches (hierarchical)
- `has_any_exact(&self, other: &GameplayTagContainer) -> bool` - Any exact match
- `has_all(&self, other: &GameplayTagContainer) -> bool` - All tags match (hierarchical)
- `has_all_exact(&self, other: &GameplayTagContainer) -> bool` - All exact matches
- `is_empty(&self) -> bool` - Check if container has no tags
- `num(&self) -> usize` - Get number of explicit tags

**Filtering:**
- `filter(&self, other: &GameplayTagContainer) -> GameplayTagContainer` - Keep only matching tags
- `matches_query(&self, query: &GameplayTagQuery) -> bool` - Complex query matching

**Iteration:**
- `gameplay_tags()` - Get reference to explicit tags vector
- `parent_tags()` - Get reference to parent tags vector

### GameplayTagCountContainer

Advanced container that maintains reference counts for each tag. Useful for stacking effects (buffs/debuffs). This is a Bevy component.

```rust
use bevy_gameplay_tag::GameplayTagCountContainer;

let mut count_container = GameplayTagCountContainer::new();
```

**Key Methods:**
- `update_tag_count(&mut self, tag: &GameplayTag, delta: i32, manager: &GameplayTagsManager, commands: &mut Commands, entity: Entity)` - Add/subtract count (triggers events)
- `set_tag_count(&mut self, tag: &GameplayTag, count: i32, manager: &GameplayTagsManager, commands: &mut Commands, entity: Entity)` - Set absolute count (triggers events)
- `get_tag_count(&self, tag: &GameplayTag) -> i32` - Get current count for a tag
- `has_matching_gameplay_tag(&self, tag: &GameplayTag) -> bool` - Check if tag exists with count > 0
- `has_all_matching_gameplay_tags(&self, tags: &GameplayTagContainer) -> bool` - Check if all tags have count > 0
- `has_any_matching_gameplay_tags(&self, tags: &GameplayTagContainer) -> bool` - Check if any tag has count > 0

**Events:**
When tag counts change, the system triggers `OnGameplayEffectTagCountChanged` events on the entity:

```rust
#[derive(Event)]
pub struct OnGameplayEffectTagCountChanged {
    pub tag: GameplayTag,
    pub tag_count: i32,
    pub event_type: GameplayTagEventType,
}

pub enum GameplayTagEventType {
    NewOrRemoved,      // Tag first added (0->1) or completely removed (1->0)
    AnyCountChanged,   // Any count change
}
```

### GameplayTagsManager

Central manager for tag registration and hierarchy. Available as Bevy resource after adding the plugin.

```rust
use bevy_gameplay_tag::GameplayTagsManager;

fn system(tags_manager: Res<GameplayTagsManager>) {
    // Manager is automatically available after adding GameplayTagsPlugin
}
```

**Key Methods:**
- `request_gameplay_tag(&mut self, tag_name: &str, world: &mut World) -> Option<GameplayTag>` - Get or create tag
- `add_tag_to_container(&self, tag: &GameplayTag, container: &mut GameplayTagContainer, world: &World)` - Add tag with parents to container

**Internal Structure:**
- Maintains a tree of `GameplayTagNode` entities
- Maps each tag to its complete container (including parents)
- Handles parent-child relationships automatically

### GameplayTagQuery

Represents a compiled query for matching tags against containers.

```rust
use bevy_gameplay_tag::{GameplayTagQuery, GameplayTagQueryExpression};

// Create from expression
let mut expr = GameplayTagQueryExpression::new();
expr.all_tags_match()
    .add_tag(GameplayTag::new("Ability.Skill"));

let query = GameplayTagQuery::new(expr);

// Helper constructors
let query = GameplayTagQuery::make_query_match_all_tags(&container);
let query = GameplayTagQuery::make_query_match_any_tags(&container);
let query = GameplayTagQuery::make_query_match_no_tags(&container);

// Test against container
if query.matches(&entity_tags) {
    // Query matched
}
```

### GameplayTagQueryExpression

Build complex logical queries for tag matching. Supports nested expressions.

```rust
use bevy_gameplay_tag::GameplayTagQueryExpression;

let mut expr = GameplayTagQueryExpression::new();

// Match any of these tags
expr.any_tags_match()
    .add_tag(GameplayTag::new("Status.Debuff.Stunned"))
    .add_tag(GameplayTag::new("Status.Debuff.Frozen"));

// Match all tags
expr.all_tags_match()
    .add_tag(GameplayTag::new("Ability.Active"))
    .add_tag(GameplayTag::new("Ability.Skill"));

// No tags match
expr.no_tags_match()
    .add_tag(GameplayTag::new("Cooldown.Skill.Fire"));

// Nested expressions
expr.any_expr_match()
    .add_expr(other_expr);
expr.all_expr_match()
    .add_expr(another_expr);
expr.no_expr_match()
    .add_expr(yet_another_expr);
```

**Expression Types:**
- `any_tags_match()` - Returns mutable reference to add tags (OR logic)
- `all_tags_match()` - Returns mutable reference to add tags (AND logic)
- `no_tags_match()` - Returns mutable reference to add tags (NOT logic)
- `any_expr_match()` - Returns mutable reference to add sub-expressions (OR logic)
- `all_expr_match()` - Returns mutable reference to add sub-expressions (AND logic)
- `no_expr_match()` - Returns mutable reference to add sub-expressions (NOT logic)

### GameplayTagRequirements

Declarative way to define tag requirements for abilities, items, etc.

```rust
use bevy_gameplay_tag::GameplayTagRequirements;

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

// Optional: Custom query expression
requirements.tag_query = Some(custom_query);

// Check if requirements are met
if requirements.requirements_met(&entity_tags) {
    // Requirements satisfied
}
```

**Fields:**
- `require_tags: GameplayTagContainer` - Tags that must be present
- `ignore_tags: GameplayTagContainer` - Tags that must NOT be present
- `tag_query: Option<GameplayTagQuery>` - Optional custom query

**Methods:**
- `new() -> Self` - Create empty requirements
- `requirements_met(&self, container: &GameplayTagContainer) -> bool` - Check if container satisfies requirements
- `is_empty(&self) -> bool` - Check if requirements are empty

## Plugin Integration

### GameplayTagsPlugin

Add to Bevy app to enable the tag system.

```rust
use bevy_gameplay_tag::GameplayTagsPlugin;

// Default initialization (uses inline JSON configuration)
app.add_plugins(GameplayTagsPlugin::default());

// Load tags from external JSON file
app.add_plugins(GameplayTagsPlugin::new("assets/tag_data.json"));
```

### GameplayTagsSettings

Resource for configuring tag data source.

```rust
use bevy_gameplay_tag::GameplayTagsSettings;

// Inline JSON data
let settings = GameplayTagsSettings {
    json_data: Some(json_string),
    data_path: None,
};

// External file path
let settings = GameplayTagsSettings {
    json_data: None,
    data_path: Some("assets/tag_data.json".to_string()),
};
```

### JSON Tag Data Format

The plugin expects JSON in this format:

```json
{
  "GameplayTagList": [
    {
      "Tag": "Ability.Skill.Fire",
      "DevComment": "Fire skill - deals fire damage"
    },
    {
      "Tag": "Status.Buff.Strength",
      "DevComment": "Increases physical damage"
    },
    {
      "Tag": "Cooldown.Skill.Fire",
      "DevComment": "Fire skill cooldown tracker"
    }
  ]
}
```

**Fields:**
- `Tag` (required): The tag name with dot notation for hierarchy
- `DevComment` (optional): Description for documentation

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
use bevy_gameplay_tag::{OnGameplayEffectTagCountChanged, GameplayTagEventType};

fn setup(mut commands: Commands) {
    let entity = commands
        .spawn((
            Name::new("Player"),
            GameplayTagCountContainer::new(),
        ))
        .id();

    commands.entity(entity).observe(on_tag_changed);
}

fn on_tag_changed(
    trigger: Trigger<OnGameplayEffectTagCountChanged>,
    query: Query<&Name>
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

### Pattern 3: Checking Conditions

```rust
fn can_use_skill(
    entity_tags: &GameplayTagCountContainer,
) -> bool {
    let cooldown_tag = GameplayTag::new("Cooldown.Skill.Fire");
    let stunned_tag = GameplayTag::new("Status.Debuff.Stunned");

    // Can use if not on cooldown and not stunned
    !entity_tags.has_matching_gameplay_tag(&cooldown_tag)
        && !entity_tags.has_matching_gameplay_tag(&stunned_tag)
}
```

### Pattern 4: Hierarchical Matching

```rust
// Entity has "Ability.Skill.Fire"
let entity_tags = /* ... */;

// These all match due to hierarchy
entity_tags.has_tag(&GameplayTag::new("Ability.Skill.Fire")); // true
entity_tags.has_tag(&GameplayTag::new("Ability.Skill"));      // true
entity_tags.has_tag(&GameplayTag::new("Ability"));            // true

// Exact match only
entity_tags.has_tag_exact(&GameplayTag::new("Ability"));      // false
entity_tags.has_tag_exact(&GameplayTag::new("Ability.Skill.Fire")); // true
```

### Pattern 5: Complex Queries

```rust
fn check_can_cast_spell(entity_tags: &GameplayTagContainer) -> bool {
    let mut expr = GameplayTagQueryExpression::new();

    // Must have mana
    expr.all_tags_match()
        .add_tag(GameplayTag::new("Resource.Mana"));

    // Must NOT be silenced or stunned
    expr.no_tags_match()
        .add_tag(GameplayTag::new("Status.Debuff.Silence"))
        .add_tag(GameplayTag::new("Status.Debuff.Stunned"));

    // Must have at least one spell
    expr.any_tags_match()
        .add_tag(GameplayTag::new("Ability.Spell.Fireball"))
        .add_tag(GameplayTag::new("Ability.Spell.IceBolt"));

    let query = GameplayTagQuery::new(expr);
    query.matches(entity_tags)
}
```

### Pattern 6: Buff Duration Tracking

```rust
// Use tag count to represent remaining duration
fn apply_timed_buff(
    entity: Entity,
    tags: &mut GameplayTagCountContainer,
    tags_manager: &GameplayTagsManager,
    commands: &mut Commands,
    duration_seconds: i32,
) {
    let buff_tag = GameplayTag::new("Status.Buff.Strength");
    tags.set_tag_count(&buff_tag, duration_seconds, tags_manager, commands, entity);
}

fn tick_buffs(
    mut query: Query<(Entity, &mut GameplayTagCountContainer)>,
    tags_manager: Res<GameplayTagsManager>,
    mut commands: Commands,
    time: Res<Time>,
) {
    for (entity, mut tags) in query.iter_mut() {
        let buff_tag = GameplayTag::new("Status.Buff.Strength");
        let current = tags.get_tag_count(&buff_tag);

        if current > 0 {
            // Decrement duration
            tags.update_tag_count(&buff_tag, -1, &tags_manager, &mut commands, entity);
        }
    }
}
```

## Performance Considerations

1. **String Interning**: Tags use `string_cache::DefaultAtom` for efficient string storage and comparison
2. **Binary Search**: Container queries use binary search (O(log n)) on sorted vectors
3. **Parent Tag Caching**: Parent tags are computed once and cached in containers
4. **Event Batching**: Tag count changes trigger events efficiently through Bevy's observer system

## Best Practices

1. **Reuse Tag Instances**: Create `GameplayTag` instances once and reuse them
   ```rust
   // Good: Create once
   const FIRE_SKILL: &str = "Ability.Skill.Fire";
   let tag = GameplayTag::new(FIRE_SKILL);

   // Avoid: Creating repeatedly in hot loops
   for _ in 0..1000 {
       let tag = GameplayTag::new("Ability.Skill.Fire"); // Wasteful
   }
   ```

2. **Use Hierarchical Matching**: Design tag hierarchies to leverage parent matching
   ```rust
   // Good: Check parent tag to match all skills
   if tags.has_tag(&GameplayTag::new("Ability.Skill")) {
       // Matches Fire, Ice, Lightning, etc.
   }
   ```

3. **Choose Right Container**: Use `GameplayTagContainer` for static tags, `GameplayTagCountContainer` for dynamic tags

4. **Observe Selectively**: Only attach observers to entities that need to react to tag changes

5. **Keep Hierarchies Shallow**: Limit tag depth to 3-4 levels for clarity and performance
