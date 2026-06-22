# Bevy Gameplay Tag

一个面向 Bevy 游戏引擎的层级化 Gameplay Tag 系统，设计灵感来自虚幻引擎的 Gameplay Tags。

[![License](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)
[![Bevy](https://img.shields.io/badge/Bevy-0.19-blue)](https://bevyengine.org)

[English](README.md) | [简体中文](README_zh.md)

## 这个 crate 提供什么

`bevy_gameplay_tag` 为游戏逻辑提供一套共享的“状态词汇表”，适合表达：技能、冷却、Buff、Debuff、阵营、AI 状态、物品分类等概念。

它主要覆盖四类能力：

- **层级标签**：例如 `Ability.Skill.Fire` 同时匹配 `Ability.Skill` 和 `Ability`
- **容器查询**：判断实体是否拥有任意/全部匹配标签
- **计数型标签**：支持同一标签被多个来源叠加或重复施加
- **声明式 requirements / query**：用统一规则表达允许/禁止条件，避免到处散落字符串判断

## 快速开始

先在 `Cargo.toml` 中添加依赖：

```toml
[dependencies]
bevy_gameplay_tag = "0.3.0"
bevy = "0.19.0"
```

加载插件并生成一个标签容器实体：

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

## 标签配置格式

当前 loader 期望的是**顶层数组** JSON。每一项包含：

- `tag_name`：完整层级标签名
- `description`：描述文本

示例 `tag_data.json`：

```json
[
  {
    "tag_name": "Ability.Skill.Fire",
    "description": "火焰技能"
  },
  {
    "tag_name": "Status.Buff.Haste",
    "description": "移动速度提升"
  },
  {
    "tag_name": "Status.Debuff.Silence",
    "description": "无法释放技能"
  }
]
```

如果你希望在构建 Bevy App 之前先做校验，可以使用：

```rust
use bevy_gameplay_tag::GameplayTagsSettings;

let rows = GameplayTagsSettings::parse_tag_table(json_source)?;
println!("加载了 {} 条标签定义", rows.len());
```

## API 地图

### 推荐主入口

- `GameplayTag` —— 单个不可变标签值
- `GameplayTagContainer` —— 集合式标签容器，支持父标签匹配
- `GameplayTagCountContainer` —— 计数/叠层型标签容器
- `GameplayTagRequirements` —— required + blocked + query 条件组合
- `GameplayTagsManager` —— 全局标签层级资源
- `GameplayTagsPlugin` —— 初始化 manager 的 Bevy 插件

### 高级入口

- `GameplayTagQuery` —— 预构建 query 对象
- `GameplayTagQueryExpression` —— 高级布尔表达式构造器
- `OnGameplayEffectTagCountChanged` —— 标签计数变化观察事件
- `GameplayTagEventType` —— 事件类型（`NewOrRemoved` / `AnyCountChanged`）

## 如何选择容器

### `GameplayTagContainer`

适合“集合式、非叠层”的标签场景。

典型场景：

- 技能分类
- 阵营/队伍标识
- 静态物品分类
- 不需要叠层的普通状态标签

```rust
use bevy_gameplay_tag::{GameplayTag, GameplayTagContainer};

let mut tags = GameplayTagContainer::new();
tags.add_tag(GameplayTag::new("Ability.Skill.Fire"), &tags_manager);

assert!(tags.has_tag(&GameplayTag::new("Ability")));
assert!(tags.has_tag(&GameplayTag::new("Ability.Skill")));
assert!(tags.has_tag_exact(&GameplayTag::new("Ability.Skill.Fire")));
```

### `GameplayTagCountContainer`

适合“同一标签可能被多个来源叠加”的场景。

典型场景：

- Buff / Debuff 层数
- 多来源冷却或限制状态
- 多个系统共同施加的禁止状态
- 可叠加的 gameplay effect

```rust
use bevy_gameplay_tag::GameplayTag;

let tag = GameplayTag::new("Status.Buff.Haste");
tag_container.update_tag_count(&tag, 3, &tags_manager, &mut commands, entity);

assert_eq!(tag_container.get_tag_count(&tag), 3);
assert!(tag_container.has_tag(&tag));
```

## 常见任务

### 添加标签并检查父标签匹配

```rust
use bevy_gameplay_tag::{GameplayTag, GameplayTagContainer};

let fire = GameplayTag::new("Ability.Skill.Fire");
let mut tags = GameplayTagContainer::new();
tags.add_tag(fire.clone(), &tags_manager);

assert!(tags.has_tag(&GameplayTag::new("Ability")));
assert!(tags.has_tag(&GameplayTag::new("Ability.Skill")));
assert!(tags.has_tag_exact(&fire));
```

### 使用计数型标签表示叠层效果

```rust
use bevy_gameplay_tag::GameplayTag;

let buff = GameplayTag::new("Status.Buff.Haste");
tag_container.update_tag_count(&buff, 1, &tags_manager, &mut commands, entity);
tag_container.update_tag_count(&buff, 1, &tags_manager, &mut commands, entity);

assert_eq!(tag_container.get_explicit_tag_count(&buff), 2);
```

### 监听标签计数变化事件

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
            println!("标签 {:?} 进入或离开了激活集合", event.tag);
        }
        GameplayTagEventType::AnyCountChanged => {
            println!("标签 {:?} 当前计数为 {}", event.tag, event.new_count);
        }
    }
}
```

### 使用声明式 requirements

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
    println!("实体可以释放火焰技能");
}

let query: GameplayTagQuery = requirements.to_query();
```

## 高级查询

对于简单场景，通常 `has_tag`、`has_any`、`has_all` 和 `GameplayTagRequirements` 就够用了。

当你需要嵌套布尔逻辑时，再使用 `GameplayTagQueryExpression`：

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
    println!("实体通过高级标签查询");
}
```

`GameplayTagQuery` 也提供了几个便捷构造器：

- `GameplayTagQuery::match_any(&container)`
- `GameplayTagQuery::match_all(&container)`
- `GameplayTagQuery::match_none(&container)`

## 错误处理与当前限制

- `GameplayTag::try_new(...)` 会先校验标签名，并拒绝空名称、前后带点、重复分隔符以及不符合 `[A-Za-z0-9_]` 规则的层级片段。
- `GameplayTagsSettings::parse_tag_table(...)` 和 `GameplayTagsSettings::load_tag_table_from_path(...)` 会校验全部行，并对非法 JSON、非法标签名、重复标签定义返回显式错误。
- `GameplayTagsPlugin` 目前仍然采用“日志式初始化”。如果插件阶段读取文件或解析 JSON 失败，会记录日志并回退到空标签表。
- 如果你需要显式处理失败，请在启动 App 之前调用 `GameplayTagsSettings::parse_tag_table(...)` 或 `GameplayTagsSettings::load_tag_table_from_path(...)`。
- 当前 crate 仍然使用运行时字符串标签，而不是代码生成常量或编译期校验体系。
- 一部分 rustdoc 示例被标记为 `ignore`，因为它们依赖一个已初始化的 `GameplayTagsManager` 运行上下文。

## 示例程序

运行内置示例：

```bash
cargo run --example example
```

示例展示了：

- 从 `examples/tag_data.json` 加载标签
- 给实体挂载 `GameplayTagCountContainer`
- 观察标签计数变化事件
- 在运行时检查层级标签匹配

## 架构概览

```text
src/
├── lib.rs                          # 模块导出
├── gameplay_tag.rs                 # 核心标签定义
├── gameplay_tags_manager.rs        # 标签加载与层级管理
├── gameplay_tag_container.rs       # 集合式标签容器与 query 表达式
├── gameplay_tag_count_container.rs # 计数型标签容器与事件
├── gameplay_tag_requirements.rs    # 声明式 requirements 封装
└── gameplay_tags_plugin.rs         # Bevy 插件集成
```

## 兼容性

| Bevy 版本 | 插件版本 |
| --------- | -------- |
| 0.19.0    | 0.3.0    |

## 许可证

本项目采用 MIT 许可证（[LICENSE](LICENSE) 或 http://opensource.org/licenses/MIT）。

## 致谢

本项目灵感来源于虚幻引擎的 Gameplay Tag 系统，并针对 Rust 与 Bevy 生态做了适配。
