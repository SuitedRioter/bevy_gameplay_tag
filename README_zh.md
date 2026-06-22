# Bevy Gameplay Tag

一个为 Bevy 游戏引擎设计的强大而灵活的层级游戏标签系统，灵感来源于虚幻引擎的 Gameplay Tag 系统。

[![License](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)
[![Bevy](https://img.shields.io/badge/Bevy-0.19-blue)](https://bevyengine.org)

[English](README.md) | [简体中文](README_zh.md)

## 特性

- **层级标签系统**：创建父子标签关系（例如：`Ability.Skill.Fire`）
- **灵活匹配**：支持精确匹配和层级匹配两种模式
- **引用计数**：跟踪标签计数并自动触发事件通知
- **复杂查询**：使用布尔逻辑构建复杂的标签查询
- **事件驱动**：使用观察者模式响应标签变化
- **JSON 配置**：在外部 JSON 文件中定义标签层级结构
- **高性能**：通过字符串驻留和二分查找优化性能
- **类型安全**：利用 Rust 的类型系统提供编译时安全保障

## 安装

在你的 `Cargo.toml` 中添加：

```toml
[dependencies]
bevy_gameplay_tag = "0.3.0"
bevy = "0.19.0"
```

## 快速开始

### 1. 添加插件

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

### 2. 定义标签（JSON）

创建一个 `tag_data.json` 文件：

```json
[
    {
        "tag_name": "Ability",
        "description": "所有技能的根标签"
    },
    {
        "tag_name": "Ability.Skill",
        "description": "技能子类别"
    },
    {
        "tag_name": "Ability.Skill.Fire",
        "description": "火焰技能"
    },
    {
        "tag_name": "Status.Buff",
        "description": "正面状态效果"
    },
    {
        "tag_name": "Status.Debuff",
        "description": "负面状态效果"
    }
]
```

在应用中加载：

```rust
use bevy_gameplay_tag::GameplayTagsPlugin;

App::new()
    .add_plugins(GameplayTagsPlugin::with_data_path(
        "assets/tag_data.json".to_string(),
    ))
    .run();
```

### 3. 在游戏中使用标签

```rust
use bevy::prelude::*;
use bevy_gameplay_tag::*;

fn setup(mut commands: Commands) {
    // 生成一个带有标签计数容器的实体
    commands.spawn(GameplayTagCountContainer::new());
}

fn add_tags_system(
    mut query: Query<(Entity, &mut GameplayTagCountContainer)>,
    tags_manager: Res<GameplayTagsManager>,
    mut commands: Commands,
) {
    for (entity, mut tag_container) in query.iter_mut() {
        let fire_skill = GameplayTag::new("Ability.Skill.Fire");

        // 添加标签（计数增加 1）
        tag_container.update_tag_count(
            &fire_skill,
            1,
            &tags_manager,
            &mut commands,
            entity,
        );

        // 检查实体是否拥有该标签
        if tag_container.has_matching_gameplay_tag(&fire_skill) {
            println!("实体拥有火焰技能！");
        }

        // 检查父标签（层级匹配）
        let ability_tag = GameplayTag::new("Ability");
        if tag_container.has_matching_gameplay_tag(&ability_tag) {
            println!("实体拥有某种技能！");
        }
    }
}
```

## 核心概念

### GameplayTag（游戏标签）

表示单个标签的基本构建块：

```rust
let tag = GameplayTag::new("Ability.Skill.Fire");

// 精确匹配
tag.matches_tag_exact(&other_tag);

// 层级匹配（Fire 可以匹配 Ability.Skill）
tag.matches_tag(&parent_tag, &tags_manager);
```

### GameplayTagContainer（标签容器）

具有查询功能的标签集合：

```rust
let mut container = GameplayTagContainer::new();

// 添加标签
container.add_tag(fire_tag, &tags_manager);
container.add_tag(ice_tag, &tags_manager);

// 查询标签
container.has_tag(&fire_tag);              // 检查标签或父标签
container.has_tag_exact(&fire_tag);        // 仅精确匹配
container.has_any(&other_container);       // 任意交集
container.has_all(&required_tags);         // 包含所有标签
```

### GameplayTagCountContainer（计数标签容器）

带有事件通知的引用计数标签：

```rust
let mut tag_container = GameplayTagCountContainer::new();

// 增加标签计数
tag_container.update_tag_count(&tag, 1, &tags_manager, &mut commands, entity);

// 减少标签计数
tag_container.update_tag_count(&tag, -1, &tags_manager, &mut commands, entity);

// 设置绝对计数
tag_container.set_tag_count(&tag, 5, &tags_manager, &mut commands, entity);

// 获取当前计数
let count = tag_container.get_tag_count(&tag);
```

### 标签变更事件

使用 Bevy 的观察者模式响应标签变化：

```rust
fn setup(mut commands: Commands) {
    let entity = commands.spawn(GameplayTagCountContainer::new()).id();

    // 观察标签变化
    commands.entity(entity).observe(on_tag_changed);
}

fn on_tag_changed(trigger: On<OnGameplayEffectTagCountChanged>) {
    let event = trigger.event();

    match event.event_type {
        GameplayTagEventType::NewOrRemoved => {
            println!("标签 {:?} 被添加或移除", event.tag);
        }
        GameplayTagEventType::AnyCountChanged => {
            println!("标签 {:?} 的计数变更为 {}", event.tag, event.new_count);
        }
    }
}
```

### 复杂查询

使用布尔逻辑构建复杂的标签查询：

```rust
// 创建查询表达式
let mut expr = GameplayTagQueryExpression::new();
expr.all_tags_match()
    .add_tag(GameplayTag::new("Ability.Skill.Fire"));

let mut blocked = GameplayTagQueryExpression::new();
blocked
    .no_tags_match()
    .add_tag(GameplayTag::new("Status.Debuff.Silence"));

let mut root = GameplayTagQueryExpression::new();
root.all_expr_match()
    .add_expr(expr)
    .add_expr(blocked);

let mut query = GameplayTagQuery::new();
query.build(root);

// 对容器进行测试
if query.matches(&container) {
    println!("实体可以施放火焰技能！");
}
```

### 标签需求

定义声明式的标签需求：

```rust
let mut require_tags = GameplayTagContainer::new();
require_tags.add_tag(GameplayTag::new("Ability.Skill"), &tags_manager);

let mut ignore_tags = GameplayTagContainer::new();
ignore_tags.add_tag(
    GameplayTag::new("Status.Debuff.Silence"),
    &tags_manager,
);

let requirements = GameplayTagRequirements::new(
    require_tags,
    ignore_tags,
    GameplayTagQuery::new(),
);

// 检查是否满足需求
if requirements.requirements_met(&entity_tags) {
    println!("可以使用技能！");
}
```

## 使用场景

### 技能系统

```rust
// 定义技能标签
let fire_skill = GameplayTag::new("Ability.Skill.Fire");
let cooldown = GameplayTag::new("Cooldown.Skill.Fire");

// 施放技能
tag_container.update_tag_count(&fire_skill, 1, &tags_manager, &mut commands, entity);
tag_container.update_tag_count(&cooldown, 1, &tags_manager, &mut commands, entity);

// 检查技能是否在冷却中
if tag_container.has_matching_gameplay_tag(&cooldown) {
    println!("技能正在冷却中！");
}
```

### Buff/Debuff 系统

```rust
// 使用引用计数叠加 Buff
let strength_buff = GameplayTag::new("Status.Buff.Strength");

// 添加 3 层
tag_container.update_tag_count(&strength_buff, 3, &tags_manager, &mut commands, entity);

// 获取层数
let stacks = tag_container.get_tag_count(&strength_buff);
println!("力量 Buff 有 {} 层", stacks);
```

### 状态机

```rust
// 将状态定义为标签
let idle = GameplayTag::new("State.Idle");
let running = GameplayTag::new("State.Running");
let jumping = GameplayTag::new("State.Jumping");

// 状态转换
tag_container.set_tag_count(&idle, 0, &tags_manager, &mut commands, entity);
tag_container.set_tag_count(&running, 1, &tags_manager, &mut commands, entity);
```

### 队伍/阵营系统

```rust
let player_team = GameplayTag::new("Teams.Player");
let monster_team = GameplayTag::new("Teams.Monster");

// 检查实体是否在同一队伍
if entity1_tags.has_any(&entity2_tags) {
    println!("同一队伍！");
}
```

### 物品系统

```rust
// 定义物品类型标签
let equipment = GameplayTag::new("Item.Type.Equipment");
let weapon = GameplayTag::new("Item.Type.Equipment.Weapon");
let consumable = GameplayTag::new("Item.Type.Consumable");

// 物品属性标签
let useable = GameplayTag::new("Item.Interaction.Useable");
let dismantleable = GameplayTag::new("Item.Interaction.Dismantleable");

// 查询所有装备类物品
if item_tags.has_tag(&equipment) {
    println!("这是一件装备");
}

// 检查物品是否可使用
if item_tags.has_tag_exact(&useable) {
    println!("物品可以使用");
}
```

### AI 行为树

```rust
// 定义 AI 状态标签
let alert = GameplayTag::new("AI.State.Alert");
let patrolling = GameplayTag::new("AI.State.Patrolling");
let chasing = GameplayTag::new("AI.State.Chasing");

// 定义感知标签
let can_see_player = GameplayTag::new("AI.Perception.CanSeePlayer");
let can_hear_player = GameplayTag::new("AI.Perception.CanHearPlayer");

// 行为树条件检查
if ai_tags.has_matching_gameplay_tag(&can_see_player)
    && !ai_tags.has_matching_gameplay_tag(&chasing) {
    // 切换到追逐状态
    ai_tags.set_tag_count(&chasing, 1, &tags_manager, &mut commands, entity);
}
```

### RPG 角色属性系统

```rust
// 定义角色职业标签
let warrior = GameplayTag::new("Class.Warrior");
let mage = GameplayTag::new("Class.Mage");
let _rogue = GameplayTag::new("Class.Rogue");

// 定义装备需求
let mut sword_require_tags = GameplayTagContainer::new();
sword_require_tags.add_tag(warrior, &tags_manager);
let sword_requirements = GameplayTagRequirements::new(
    sword_require_tags,
    GameplayTagContainer::new(),
    GameplayTagQuery::new(),
);

// 检查角色是否可以装备
if sword_requirements.requirements_met(&character_tags) {
    println!("战士可以装备这把剑");
}

// 定义技能学习需求
let mut fireball_require_tags = GameplayTagContainer::new();
fireball_require_tags.add_tag(mage, &tags_manager);
fireball_require_tags.add_tag(GameplayTag::new("Level.10"), &tags_manager);
let fireball_requirements = GameplayTagRequirements::new(
    fireball_require_tags,
    GameplayTagContainer::new(),
    GameplayTagQuery::new(),
);

if fireball_requirements.requirements_met(&character_tags) {
    println!("法师达到 10 级，可以学习火球术");
}
```

## 性能优化

- **字符串驻留**：使用 `string_cache` 实现高效的字符串存储和比较
- **二分查找**：在排序容器中实现 O(log n) 的标签查找
- **延迟更新**：仅在必要时更新父标签
- **高效计数**：基于 HashMap 的引用计数

## 示例

查看 [examples](examples/) 目录获取完整的工作示例：

```bash
cargo run --example example
```

## 兼容性

| Bevy 版本 | 插件版本 |
| --------- | -------- |
| 0.19.0    | 0.3.0    |

## 许可证

本项目采用 MIT 许可证（[LICENSE](LICENSE) 或 http://opensource.org/licenses/MIT）。

## 致谢

本项目灵感来源于虚幻引擎的 Gameplay Tag 系统，并针对 Rust 和 Bevy 生态系统进行了改编和优化。
