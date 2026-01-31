# Bevy Gameplay Tag

一个为 Bevy 游戏引擎设计的强大而灵活的层级游戏标签系统，灵感来源于虚幻引擎的 Gameplay Tag 系统。

[![Crates.io](https://img.shields.io/crates/v/bevy_gameplay_tag.svg)](https://crates.io/crates/bevy_gameplay_tag)
[![License](https://img.shields.io/badge/license-MIT%2FApache-blue.svg)](https://github.com/yourusername/bevy_gameplay_tag)
[![Bevy](https://img.shields.io/badge/Bevy-0.18-blue)](https://bevyengine.org)

[English](README.md) | 简体中文

## 特性

- **层级标签系统**：创建父子标签关系（例如：`Ability.Skill.Fire`）
- **灵活匹配**：支持精确匹配和层级匹配两种模式
- **引用计数**：跟踪标签计数并自动触发事件通知
- **复杂查询**：使用布尔逻辑构建复杂的标签查询
- **事件驱动**：使用观察者模式响应标签变化
- **JSON 配置**：在外部 JSON 文件中定义标签层级结构
- **高性能**：通过字符串驻留和二分查找优化性能
- **类型安全**：利用 Rust 的类型系统提供编译时安全保障
- **Claude Code支持**：当其他开发者在他们的项目中使用 Claude Code 时，如果他们安装了这个 skill，让 Claude Code 在处理 bevy_gameplay_tag 相关问题时提供更准确、更专业的帮助！

## 安装

在你的 `Cargo.toml` 中添加：

```toml
[dependencies]
bevy_gameplay_tag = "0.1.0"
bevy = "0.18"
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
{
    "GameplayTagList": [
        {
            "Tag": "Ability",
            "DevComment": "所有技能的根标签"
        },
        {
            "Tag": "Ability.Skill",
            "DevComment": "技能子类别"
        },
        {
            "Tag": "Ability.Skill.Fire",
            "DevComment": "火焰技能"
        },
        {
            "Tag": "Status.Buff",
            "DevComment": "正面状态效果"
        },
        {
            "Tag": "Status.Debuff",
            "DevComment": "负面状态效果"
        }
    ]
}
```

在应用中加载：

```rust
use bevy_gameplay_tag::{GameplayTagsPlugin, GameplayTagsSettings};

App::new()
    .add_plugins(GameplayTagsPlugin::new("assets/tag_data.json"))
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

fn on_tag_changed(trigger: Trigger<OnGameplayEffectTagCountChanged>) {
    let event = trigger.event();

    match event.event_type {
        GameplayTagEventType::NewOrRemoved => {
            println!("标签 {} 被添加或移除", event.tag);
        }
        GameplayTagEventType::AnyCountChanged => {
            println!("标签 {} 的计数变更为 {}", event.tag, event.tag_count);
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
expr.no_tags_match()
    .add_tag(GameplayTag::new("Status.Debuff.Silence"));

let query = GameplayTagQuery::new(expr);

// 对容器进行测试
if query.matches(&container) {
    println!("实体可以施放火焰技能！");
}
```

### 标签需求

定义声明式的标签需求：

```rust
let mut requirements = GameplayTagRequirements::new();

// 必须拥有这些标签
requirements.require_tags.add_tag(
    GameplayTag::new("Ability.Skill"),
    &tags_manager
);

// 不能拥有这些标签
requirements.ignore_tags.add_tag(
    GameplayTag::new("Status.Debuff.Silence"),
    &tags_manager
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
let rogue = GameplayTag::new("Class.Rogue");

// 定义装备需求
let mut sword_requirements = GameplayTagRequirements::new();
sword_requirements.require_tags.add_tag(warrior, &tags_manager);

// 检查角色是否可以装备
if sword_requirements.requirements_met(&character_tags) {
    println!("战士可以装备这把剑");
}

// 定义技能学习需求
let mut fireball_requirements = GameplayTagRequirements::new();
fireball_requirements.require_tags.add_tag(mage, &tags_manager);
fireball_requirements.require_tags.add_tag(
    GameplayTag::new("Level.10"),
    &tags_manager
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
| 0.18      | 0.1.0    |

## 架构设计

### 模块结构

```
src/
├── lib.rs                          # 模块导出
├── gameplay_tag.rs                 # 核心标签定义
├── gameplay_tags_manager.rs        # 标签管理器
├── gameplay_tag_container.rs       # 标签容器和查询系统
├── gameplay_tag_count_container.rs # 引用计数标签容器
├── gameplay_tag_requirements.rs    # 标签需求系统
└── gameplay_tags_plugin.rs         # Bevy 插件集成
```

### 设计理念

1. **层级结构**：标签使用点号分隔的层级命名（如 `A.B.C`），自动建立父子关系
2. **引用计数**：支持标签的叠加效果，适用于 Buff/Debuff 等需要计数的场景
3. **事件驱动**：标签变化时自动触发事件，便于其他系统响应
4. **查询灵活**：支持精确匹配、层级匹配、布尔逻辑组合等多种查询方式
5. **ECS 集成**：完全基于 Bevy 的 ECS 架构，充分利用组件系统的优势

## 最佳实践

### 标签命名规范

建议使用清晰的层级命名结构：

```
根类别.子类别.具体项
例如：
- Ability.Skill.Fire
- Status.Buff.Strength
- Item.Type.Equipment.Weapon
- AI.State.Patrolling
```

### 标签组织建议

1. **按功能分类**：将相关标签组织在同一层级下
2. **避免过深层级**：建议不超过 4-5 层
3. **使用有意义的名称**：标签名应该自解释
4. **保持一致性**：在整个项目中使用统一的命名风格

### 性能建议

1. **预加载标签**：在游戏启动时通过 JSON 加载所有标签定义
2. **复用容器**：避免频繁创建和销毁标签容器
3. **批量操作**：尽可能批量添加或移除标签
4. **合理使用事件**：只在必要时监听标签变化事件

## 与虚幻引擎的对比

如果你熟悉虚幻引擎的 Gameplay Tag 系统，这里是一些对应关系：

| 虚幻引擎                  | bevy_gameplay_tag         |
| ------------------------- | ------------------------- |
| FGameplayTag              | GameplayTag               |
| FGameplayTagContainer     | GameplayTagContainer      |
| FGameplayTagQuery         | GameplayTagQuery          |
| UGameplayTagsManager      | GameplayTagsManager       |
| GameplayTagCountContainer | GameplayTagCountContainer |

主要区别：

- 使用 Rust 的所有权系统替代 UE 的智能指针
- 基于 Bevy ECS 而非 UObject 系统
- 使用 JSON 配置而非 .ini 文件
- 事件系统基于 Bevy 的 Observer 模式

## 贡献

欢迎贡献！请随时提交 Pull Request。

### 开发指南

```bash
# 克隆仓库
git clone https://github.com/yourusername/bevy_gameplay_tag.git
cd bevy_gameplay_tag

# 运行测试
cargo test

# 运行示例
cargo run --example example

# 检查代码格式
cargo fmt --check

# 运行 Clippy
cargo clippy
```

## 许可证

本项目采用以下任一许可证：

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) 或 http://www.apache.org/licenses/LICENSE-2.0)
- MIT license ([LICENSE-MIT](LICENSE-MIT) 或 http://opensource.org/licenses/MIT)

由你选择。

## 致谢

本项目灵感来源于虚幻引擎的 Gameplay Tag 系统，并针对 Rust 和 Bevy 生态系统进行了改编和优化。

## 资源链接

- [在线文档](https://docs.rs/bevy_gameplay_tag)
- [示例代码](examples/)
- [Bevy 引擎](https://bevyengine.org)
- [虚幻引擎 Gameplay Tags 文档](https://docs.unrealengine.com/en-US/gameplay-tags-in-unreal-engine/)

## 常见问题

### Q: 标签的层级匹配是如何工作的？

A: 当你查询一个父标签时，所有拥有该父标签的子标签都会匹配。例如，如果实体有 `Ability.Skill.Fire` 标签，查询 `Ability` 或 `Ability.Skill` 都会返回 true。

### Q: 什么时候使用 GameplayTagContainer vs GameplayTagCountContainer？

A: 如果你只需要简单的标签存在性检查，使用 `GameplayTagContainer`。如果需要跟踪标签的数量（如 Buff 层数）或需要事件通知，使用 `GameplayTagCountContainer`。

### Q: 如何动态添加新标签？

A: 标签在首次使用时会自动注册到 `GameplayTagsManager`。你也可以通过 JSON 文件预定义所有标签。

### Q: 性能如何？

A: 系统使用字符串驻留和二分查找优化，标签查询的时间复杂度为 O(log n)。对于大多数游戏场景，性能表现优秀。

### Q: 可以在运行时修改标签层级结构吗？

A: 标签的层级关系在创建时确定。虽然可以动态添加新标签，但不建议在运行时修改已有标签的层级关系。
