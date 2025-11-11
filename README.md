# bevy_gameplay_tag：为 Bevy 引擎打造的游戏标签系统

## 项目概述
`bevy_gameplay_tag` 是一个专为 Bevy 游戏引擎设计的 Gameplay Tag 系统实现。该项目受到虚幻引擎（Unreal Engine）的 Gameplay Tag 系统启发，为 Rust 游戏开发者提供了一套强大而灵活的标签管理机制。

## 核心架构
### 1. GameplayTag - 标签的基础表示
项目的核心是 `GameplayTag` 结构体，它使用 `string_cache` 库的 `DefaultAtom` 类型来存储标签名称，这确保了标签的高效存储和比较。
每个标签都支持层级结构，例如 `A.B.C` 这样的命名方式，其中 `A` 是 `A.B` 的父标签，`A.B` 是 `A.B.C` 的父标签。这种设计允许开发者创建具有继承关系的标签体系。
   
### 2. GameplayTagsManager - 标签管理中心
`GameplayTagsManager` 是整个系统的核心管理器，负责标签的注册、存储和查询。管理器使用树形结构存储标签节点，通过 Bevy 的 ECS 系统将标签以实体-组件的方式组织。它维护了一个 tag_map，将每个标签映射到包含该标签及其所有父标签的完整容器。
标签数据可以通过 JSON 文件或直接通过代码配置加载，提供了灵活的配置方式。

### 3. GameplayTagContainer - 标签集合
`GameplayTagContainer` 是标签的容器类，可以存储多个标签。它内部维护两个集合：`gameplay_tags`（显式标签）和 `parent_tags`（父标签）。
容器提供了丰富的查询和操作方法，包括：
* 精确匹配和层级匹配
* 任意匹配和全部匹配
* 标签的添加、删除和过滤

所有标签在容器中都是排序存储的，使用二分查找实现高效的查询操作。

### 4. GameplayTagCountContainer - 带计数的标签容器
这是一个更高级的容器实现，为每个标签维护引用计数。这在处理临时效果、Buff/Debuff 等游戏机制时特别有用。 当标签计数发生变化时，系统会自动触发事件，允许其他系统响应这些变化。
事件系统区分了两种类型的变化：

* `NewOrRemoved`：标签首次添加或完全移除时触发
* `AnyCountChanged`：任何计数变化时触发

## 核心功能特性
### 1. 层级匹配机制
标签支持两种匹配方式：

* 精确匹配（Exact Match）：只匹配完全相同的标签
* 层级匹配（Hierarchical Match）：子标签可以匹配父标签

例如，如果一个实体有 `A.B.C` 标签，那么查询 `A` 或 `A.B` 时都会匹配成功。

### 2. 查询表达式系统
项目实现了一套强大的查询表达式系统 `GameplayTagQueryExpression`，支持复杂的逻辑组合：

* `AnyTagsMatch`：匹配任意标签
* `AllTagsMatch`：匹配所有标签
* `NoTagsMatch`：不匹配任何标签
* 表达式的嵌套组合

查询表达式可以递归匹配，支持构建复杂的条件逻辑。

### 3. 标签需求系统
`GameplayTagRequirements` 提供了一种声明式的方式来定义标签要求，包括：

* 必需标签（require_tags）
* 排除标签（ignore_tags）
* 自定义查询表达式

这对于实现技能系统、状态机等游戏机制非常有用。

### 4. Bevy ECS 集成
项目提供了 `GameplayTagsPlugin`，可以无缝集成到 Bevy 应用中。标签容器可以作为组件附加到实体上，利用 Bevy 的 Observer 模式实现事件监听。

## 技术实现亮点
### 1. 性能优化
* 使用 `string_cache` 库优化字符串存储和比较
* 所有容器内部使用排序数组和二分查找，确保 O(log n) 的查询复杂度  
* 父标签的延迟更新机制，减少不必要的重建操作

### 2. 类型安全
项目充分利用 Rust 的类型系统，确保标签操作的安全性。所有公共 API 都经过精心设计，避免了常见的运行时错误。

### 3. 灵活的配置系统
标签可以通过 JSON 文件定义，也可以在代码中动态创建。默认配置提供了示例数据，方便快速上手。

## 应用场景
`bevy_gameplay_tag` 适用于多种游戏开发场景：

1. 技能系统：使用标签表示技能类型、CD状态、施放条件等
2. Buff/Debuff系统：通过标签计数管理各种效果的叠加
3. 状态机：用标签表示游戏对象的各种状态
4. AI系统：标签可以用于行为树的条件判断
5. 物品系统：使用标签分类和查询物品属性

## 依赖关系
项目依赖于以下核心库：

* `bevy 0.17.2`：游戏引擎框架
* `string_cache`：高效的字符串缓存
* `serde` 和 `serde_json`：JSON 序列化支持

## Notes
这个项目是一个完整的 Gameplay Tag 系统实现，借鉴了虚幻引擎的设计理念，但针对 Rust 和 Bevy 生态进行了适配和优化。项目结构清晰，代码质量高，提供了丰富的 API 文档注释。

主要模块包括：

* `gameplay_tag.rs`：标签的基础定义
* `gameplay_tags_manager.rs`：标签管理器
* `gameplay_tag_container.rs`：标签容器和查询系统
* `gameplay_tag_count_container.rs`：带计数的标签容器
* `gameplay_tag_requirements.rs`：标签需求系统
* `gameplay_tags_plugin.rs`：Bevy 插件集成

项目特别适合需要复杂标签管理和查询机制的游戏项目，如 RPG、MOBA、RTS 等类型的游戏。通过合理使用这个系统，可以大大简化游戏逻辑的实现，提高代码的可维护性和扩展性。