# rgpui-character

基于 [rgpui](https://github.com/launcher-rs/rgpui) 框架的桌宠与 UI 角色运行时系统。

```
角色 = 行为 + 动画 + 物理 + 渲染
```

## 架构

| 模块 | 说明 |
|------|------|
| `core` | 核心数据模型：`Vec2`、`Rect`、`TextureId`、`CharacterState`、`Character` |
| `animation` | 精灵/片段动画：`AnimationClip`、`AnimationPlayer`、`AnimationState` |
| `behavior` | 可插拔行为：`Behavior` trait、`IdleBehavior`、`ConstantMoveBehavior` |
| `physics` | 轻量物理：`PhysicsConfig`、摩擦、边界、弹跳 |
| `asset` | 资源管理：`AssetManager`、纹理与动画查找 |
| `render` | 渲染抽象：`RenderCommand`、`RenderBackend` trait |
| `runtime` | 多角色调度器：`CharacterRuntime`、`CharacterEvent` |

**每帧数据流：**

```
Behavior::update() → BehaviorAction → apply_action()
  → update_physics()（位置、速度、摩擦、边界）
  → AnimationState::update()（推进帧）
  → Character::build_render() → Option<RenderCommand>
  → Vec<RenderCommand>（输出到渲染层）
```

## 使用

在 `Cargo.toml` 中添加：

```toml
[dependencies]
rgpui-character = { git = "https://github.com/launcher-rs/rgpui-character.git" }
```

基本示例：

```rust
use rgpui_character::*;

let mut runtime = CharacterRuntime::new();
runtime.assets.register_animation(AnimationClip::new(
    "idle",
    vec![TextureId::new("idle-0")],
    1.0,
    true,
));

let mut character = Character::new("pet");
character.animation.play("idle");
runtime.add_character(character);

// 每帧更新
let commands = runtime.update(dt, &mut my_behavior);

// 将渲染命令交给后端
for cmd in &commands {
    backend.draw_sprite(cmd);
}
```

完整桌宠应用请参考 [desktop_pet 示例](examples/desktop_pet.rs)，包含透明覆盖窗口、鼠标穿透、系统托盘和动画活动。

## 设计思想

- **库本身不依赖 rgpui** — 仅通过 `RenderBackend` trait 输出 `RenderCommand`，与具体渲染器解耦。
- **行为可插拔** — 通过 `Behavior` trait 实现，未来可接入行为树、脚本或 AI。
- **轻量物理** — 位置/速度/摩擦/边界，适用于桌宠场景，无需完整物理引擎。
- **v0.1 基于精灵** — 未来版本可能添加分层伪 Live2D 骨骼系统。
- **不使用 ECS** — 采用简单的线性 `O(n)` 角色遍历。

## 运行示例

```bash
cargo run --example desktop_pet
```

## 许可证

Apache License, Version 2.0
