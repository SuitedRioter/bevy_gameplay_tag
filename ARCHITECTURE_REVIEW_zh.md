# bevy_gameplay_tag 基础标签库架构评审与重构建议

## 1. 文档目标

本文档从 **Bevy 基础标签库** 的视角，对当前项目进行架构评价，并给出一套 **最小改动、渐进式** 的重构建议。

这里不把项目当成完整 gameplay framework 来评价，而是把它当成一个可复用的基础库，重点关注：

- 核心抽象是否成立
- 核心数据结构是否足够纯
- Bevy 集成是否自然但不过度侵入
- 数据更新与副作用是否分层清晰
- 项目是否适合继续演进为长期维护的基础库

## 2. 总体结论

当前项目的整体判断可以概括为：

> 这是一个 **方向正确、已经可用，但核心层还不够“轻”和“纯”** 的 Bevy 基础标签库。

它的优点在于：

- 已经实现了真正有价值的层级标签语义，而不只是字符串集合
- `GameplayTag` / `GameplayTagContainer` / `GameplayTagCountContainer` / `GameplayTagsManager` 的核心概念基本成立
- 同时支持普通标签集合与计数型标签集合，适合游戏中的 buff、debuff、叠层状态等场景
- 具备一定的查询能力、父标签展开能力和事件通知能力

它当前的主要问题不在于“功能不够”，而在于“库边界不够收敛”：

- 核心容器类型对 `Res<T>`、`Commands`、`World` 等 Bevy ECS 类型耦合偏重
- 数据修改和副作用触发混在一起
- registry、配置加载、debug world tree 的职责边界不够清晰
- 某些 API 更像框架内部调用接口，不像基础库核心接口

## 3. 基础标签库视角下的优点

### 3.1 核心领域抽象是成立的

当前项目已经形成了较稳定的概念层：

- `GameplayTag`：单个标签值对象
- `GameplayTagContainer`：标签集合与层级匹配容器
- `GameplayTagCountContainer`：带计数能力的标签容器
- `GameplayTagsManager`：全局标签定义与父级关系查询入口

这套结构说明项目不是功能堆砌，而是在围绕 gameplay tag 领域建模。

### 3.2 层级标签能力是项目最大价值点

项目支持诸如 `A.B.C` 自动具备 `A`、`A.B` 父标签语义，这一点是基础标签库最核心的能力之一。

相关实现位于：

- `src/gameplay_tags_manager.rs`
- `src/gameplay_tag_container.rs`
- `src/gameplay_tag_count_container.rs`

这说明该项目已经超越了普通的 `HashSet<String>` 方案，具备真正的 gameplay tag 语义。

### 3.3 普通容器与计数容器并存，设计合理

很多游戏场景不仅需要“有/没有标签”，还需要“标签有多少层”。

当前项目区分了：

- `GameplayTagContainer`
- `GameplayTagCountContainer`

这是合理且实用的，因为它可以覆盖：

- 技能状态
- Buff/Debuff 叠层
- 条件门槛
- 计数型效果触发

### 3.4 API 基本可理解、可使用

例如：

- `has_tag`
- `has_tag_exact`
- `has_any`
- `has_all`
- `update_tag_count`
- `set_tag_count`

这些接口命名基本直观，说明对外可用性是被考虑过的。

## 4. 基础标签库视角下的主要问题

### 4.1 核心层被 Bevy System 参数类型污染

当前多个核心 API 直接使用：

- `&Res<GameplayTagsManager>`
- `&mut Commands`
- `&mut World`

这使得本该偏“纯逻辑”的标签容器层，与 Bevy ECS 的运行时细节绑得过紧。

典型位置：

- `src/gameplay_tag_container.rs`
- `src/gameplay_tag_count_container.rs`

这会带来几个问题：

- 不利于单元测试
- 不利于在非 system 场景中复用
- 不利于后续将 core 层与 bevy 适配层拆分
- 使用者在调用时需要感知太多 Bevy 上下文

对于基础库而言，理想状态应当是：

> 核心类型依赖普通引用或抽象 registry，而不是依赖 Bevy system parameter 包装类型。

### 4.2 数据修改和副作用混在一起

`GameplayTagCountContainer` 当前不仅负责维护标签计数，还负责：

- 触发事件
- 依赖 `Commands`
- 依赖 `Entity`
- 甚至在 `reset` 中直接操作 `World`

这使它更像一个“带副作用的服务对象”，而不是基础库中的核心数据结构。

从基础库视角看，更理想的分层是：

- 核心层只负责改数据、计算变化
- Bevy 层负责根据变化发事件、触发 observer、做日志或同步

### 4.3 `GameplayTagsManager` 职责偏重

当前 `GameplayTagsManager` 同时承担了：

- 配置读取/JSON 解析入口
- 层级标签 registry
- ECS world tree 构造者

这说明它既像 registry，又像 loader，又像 debug tree builder。

对基础库而言，这种职责混合会让类型边界不够清晰，也不利于未来扩展。

### 4.4 某些类型角色不够清晰

例如 `GameplayTagContainer` 目前既是值对象容器，又直接 `derive(Component)`。

这未必是错误，但会弱化它作为“标签集合值对象”的角色，使后续 API 风格更容易继续混杂：

- 一部分像纯数据容器
- 一部分像 ECS 组件
- 一部分像运行时工具对象

对于基础库来说，类型角色越明确，长期维护越轻松。

## 5. 建议保留的设计

以下设计方向总体是对的，不建议推翻。

### 5.1 保留 `GameplayTag`

作为标签值对象，它是整个库的核心入口，应该继续保留。

### 5.2 保留层级标签语义

项目最重要的价值就是层级匹配语义，这部分不但应保留，还应作为核心卖点继续强化。

### 5.3 保留“普通容器 + 计数容器”双层设计

这套设计非常契合游戏常见场景，不建议收缩为单一容器。

### 5.4 保留事件语义，但调整触发层级

`NewOrRemoved` 和 `AnyCountChanged` 的事件粒度设计是合理的。

建议保留事件语义，但将触发动作从核心容器内部逐步迁移到 Bevy 集成层。

## 6. 最小改动重构目标

这一轮重构的目标不是推翻现有代码，而是做一次 **库边界收敛**：

1. 让核心 API 更像基础库 API
2. 让 Bevy 运行时细节从核心层退出
3. 让数据修改和副作用逐步分离
4. 为后续 core/bevy 分层重构打基础

## 7. 最小改动重构清单

下面给出基于当前代码结构的最小改动方案。

### 7.1 第一阶段：去除核心层对 `Res<T>` 的直接依赖

#### 文件：`src/gameplay_tag_container.rs`

建议将以下方法的参数类型从：

```rust
&Res<GameplayTagsManager>
```

替换为：

```rust
&GameplayTagsManager
```

涉及的方法包括但不限于：

- `add_tag`
- `add_tag_fast`
- `add_parent_tag`
- `fill_parent_tags`
- `remove_tag`
- `remove_tags`
- `append_matches_tags`
- `append_tags`
- `filter`
- `filter_exact`

这样做的好处是：

- 核心容器 API 更纯
- 降低 Bevy system parameter 对领域层的渗透
- 后续更容易拆出纯核心层

#### 文件：`src/gameplay_tag_count_container.rs`

同样建议将以下函数中出现的：

```rust
&Res<GameplayTagsManager>
```

统一替换为：

```rust
&GameplayTagsManager
```

涉及：

- `update_tag_container_count`
- `update_tag_count`
- `update_tag_count_deferred_parent_removal`
- `set_tag_count`
- `fill_parent_tags`
- `update_tag_map_internal`
- `update_tag_map_deferred_parent_removal_internal`
- `update_explicit_tags`
- `gather_tag_change_delegates`

这一阶段改动收益很高，风险相对很低，应优先执行。

### 7.2 第二阶段：拆分 `reset` 的数据职责与 World 副作用

#### 文件：`src/gameplay_tag_count_container.rs`

当前 `reset` 同时做了：

1. 清空标签数据
2. 遍历 observer 并从 world 中移除 `Observer` 组件

建议拆为两部分：

#### 核心层保留的数据重置

```rust
pub fn reset(&mut self) {
    self.explicit_tag_count_map.clear();
    self.explicit_tags.reset();
    self.gameplay_tag_count_map.clear();
}
```

#### Bevy 辅助层处理 observer 清理

例如新增一个辅助函数：

```rust
pub fn cleanup_observers_for_entity(world: &mut World, entity: Entity)
```

这样做的好处是：

- `reset` 重新回归“只处理自身数据”的职责
- observer 生命周期管理留在 Bevy 层
- API 语义更加清晰

### 7.3 第三阶段：先把“变化计算”和“事件触发”在实现层分开

#### 文件：`src/gameplay_tag_count_container.rs`

当前这一阶段不要求立刻改变全部对外 API，可以先新增一个内部纯函数，例如：

```rust
fn collect_tag_count_changes(
    &mut self,
    tag: &GameplayTag,
    count_delta: i32,
    tags_manager: &GameplayTagsManager,
) -> Vec<TagCountChange>
```

并新增变化结构：

```rust
pub struct TagCountChange {
    pub tag: GameplayTag,
    pub old_count: i32,
    pub new_count: i32,
    pub significant: bool,
}
```

然后保留当前对外兼容接口，在内部改成：

1. 先收集变化
2. 再根据变化触发 event

这样可以在不立刻打破现有调用方式的情况下，把“纯变化计算”和“Bevy 副作用”先分开。

### 7.4 第四阶段：为 `GameplayTagsManager` 增加更纯的 registry 接口

#### 文件：`src/gameplay_tags_manager.rs`

当前 `GameplayTagsManager` 的职责偏多，且现有接口更像内部实现接口。

建议在不删除旧接口的前提下，先新增更偏 registry 语义的接口，例如：

```rust
pub fn contains_tag(&self, tag: &GameplayTag) -> bool
pub fn get_complete_tag_container(&self, tag: &GameplayTag) -> Option<&GameplayTagContainer>
pub fn get_tag_parents(&self, tag: &GameplayTag) -> Option<&[GameplayTag]>
```

这样做可以逐步引导容器层依赖更纯的 registry 接口，而不是依赖 manager 的内部实现细节。

### 7.5 第五阶段：修复 `GameplayTagRequirements::is_empty()`

#### 文件：`src/gameplay_tag_requirements.rs`

当前 `is_empty()` 的语义实现为：

```rust
has_require && has_ignore && has_query
```

这与通常对 `is_empty()` 的语义理解不一致。

更合理的实现大概率应为：

```rust
!has_require && !has_ignore && !has_query
```

这个问题不只是“风格问题”，更可能影响逻辑判断正确性，因此建议尽早修复。

### 7.6 第六阶段：优化示例代码，传达基础库推荐用法

#### 文件：`examples/example.rs`

当前示例的功能演示是有效的，但从基础库角度看，示例更像“演示逻辑能跑”，而不是“传达推荐用法”。

建议最小改动地优化：

- 用 `Player` / `Enemy` marker component 替代 `Name == "Player"` 这种筛选方式
- 将示例拆成“普通标签容器用法”和“计数容器 + 事件用法”两部分
- 尽量避免在示例中放大对复杂 ECS 上下文参数的依赖感

示例代码是用户理解库设计意图的重要入口，因此这一步很有价值。

## 8. 建议的模块演进方向

虽然这一轮建议强调“最小改动”，但从长期看，项目可以朝下面的结构演进：

```text
src/
  core/
    gameplay_tag.rs
    gameplay_tag_container.rs
    gameplay_tag_count_container.rs
    gameplay_tag_query.rs
    gameplay_tag_requirements.rs
    gameplay_tag_registry.rs

  bevy/
    plugin.rs
    events.rs
    observers.rs
    systems.rs
    resources.rs

  io/
    json_loader.rs

  debug/
    tag_tree.rs
```

其中：

- `core/`：尽量纯逻辑
- `bevy/`：只处理 Bevy 集成
- `io/`：处理配置加载
- `debug/`：处理调试树和可视化

这不是当前必须立刻完成的工作，但可作为未来演进蓝图。

## 9. 建议执行顺序

### P0：立即做

1. 修复 `GameplayTagRequirements::is_empty()`
2. 将核心 API 中所有 `&Res<GameplayTagsManager>` 改为 `&GameplayTagsManager`

### P1：下一步做

3. 拆分 `GameplayTagCountContainer::reset`
4. 提取 `collect_tag_count_changes` 内部纯函数
5. 新增 `TagCountChange`

### P2：随后做

6. 为 `GameplayTagsManager` 增加更纯的 registry 接口
7. 重写 `examples/example.rs`，让示例更能代表基础库推荐用法

### P3：可选优化

8. 将 `GameplayTagNode` 拆到 debug/internal 模块
9. 将来再考虑 `GameplayTagsManager -> GameplayTagRegistry` 的重命名
10. 视兼容性情况决定 `GameplayTagContainer` 是否继续保留 `Component`

## 10. 最终结论

如果从 **Bevy 基础标签库** 的标准来评价当前项目，可以给出如下判断：

- **方向是对的**：它已经具备真正的层级标签系统核心能力
- **当前是可用的**：不是概念验证，而是一个已具备实践价值的库
- **仍需收敛边界**：目前最需要的不是继续堆功能，而是让核心层更纯、让 Bevy 集成更分层

换句话说，这个项目已经像一个“好用的标签功能实现”，但要成长为“成熟的基础库”，还需要完成以下转变：

> 从“Bevy 中可用的标签实现”
> 逐步演进为
> “核心层纯净、适配层清晰、长期可复用的 Bevy 基础标签库”。

## 11. 相关文件参考

本文档分析时重点参考了以下文件：

- `src/gameplay_tags_plugin.rs`
- `src/gameplay_tags_manager.rs`
- `src/gameplay_tag_container.rs`
- `src/gameplay_tag_count_container.rs`
- `src/gameplay_tag_requirements.rs`
- `examples/example.rs`
