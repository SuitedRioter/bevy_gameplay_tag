use bevy::prelude::*;
use bevy_gameplay_tag::{
    gameplay_tag, gameplay_tag_names, GameplayTagCountContainer, GameplayTagEventType,
    GameplayTagsManager, GameplayTagsPlugin, OnGameplayEffectTagCountChanged,
};
use bevy_inspector_egui::{bevy_egui::EguiPlugin, quick::WorldInspectorPlugin};

/// Tag name constants — keep all tag strings in one place.
/// Use `tags_manager.get_tag(tags::DAMAGED)` for a registry-validated lookup.
mod tags {
    use super::gameplay_tag_names;

    gameplay_tag_names! {
        pub DAMAGED = "Status.Damaged";
        pub BUFF_STRENGTH = "Buff.Strength";
    }
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(EguiPlugin::default())
        .add_plugins(WorldInspectorPlugin::new())
        .add_plugins(GameplayTagsPlugin::with_data_path(
            "examples/tag_data.json",
        ))
        .add_systems(Startup, (setup, print_controls))
        .add_systems(
            Update,
            (apply_damage_system, apply_buff_system, display_tag_counts),
        )
        .run();
}

fn print_controls() {
    info!("=== 按键说明 ===");
    info!("  Space  — 给 Player 添加 1 层 Status.Damaged");
    info!("  B      — 将 Player 的 Buff.Strength 设置为 3 层");
    info!("  R      — 减少 1 层 Buff.Strength");
    info!("  I      — 打印当前标签计数");
    info!("================");
}

fn setup(mut commands: Commands) {
    let player = commands
        .spawn((Name::new("Player"), GameplayTagCountContainer::new()))
        .id();
    commands.entity(player).observe(on_player_tag_changed);

    let enemy = commands
        .spawn((Name::new("Enemy"), GameplayTagCountContainer::new()))
        .id();
    commands.entity(enemy).observe(on_enemy_tag_changed);

    commands.spawn((
        Name::new("Camera"),
        Camera3d::default(),
        Transform::from_xyz(-2.0, 2.5, 5.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));
}

fn on_player_tag_changed(trigger: On<OnGameplayEffectTagCountChanged>, query: Query<&Name>) {
    let event = trigger.event();
    let name = query.get(event.entity).unwrap();

    match event.event_type {
        GameplayTagEventType::NewOrRemoved => {
            if event.new_count > 0 {
                info!("{} 获得新标签: {:?}", name, event.tag);
            } else {
                info!("{} 失去标签: {:?}", name, event.tag);
            }
        }
        GameplayTagEventType::AnyCountChanged => {
            info!("{} 标签 {:?} 计数变更为: {}", name, event.tag, event.new_count);
        }
    }
}

fn on_enemy_tag_changed(trigger: On<OnGameplayEffectTagCountChanged>, query: Query<&Name>) {
    let event = trigger.event();
    let name = query.get(event.entity).unwrap();

    if event.event_type == GameplayTagEventType::NewOrRemoved {
        info!("敌人 {} 标签状态变化: {:?} -> {}", name, event.tag, event.new_count);
    }
}

fn apply_damage_system(
    mut query: Query<(Entity, &Name, &mut GameplayTagCountContainer)>,
    tags_manager: Res<GameplayTagsManager>,
    mut commands: Commands,
    keyboard: Res<ButtonInput<KeyCode>>,
) {
    if keyboard.just_pressed(KeyCode::Space) {
        let damage_tag = gameplay_tag!(tags::DAMAGED);
        for (entity, name, mut tag_container) in query.iter_mut() {
            if name.as_str() == "Player" {
                info!("玩家受到伤害!");
                tag_container.update_tag_count(&damage_tag, 1, &tags_manager, &mut commands, entity);
            }
        }
    }
}

fn apply_buff_system(
    mut query: Query<(Entity, &Name, &mut GameplayTagCountContainer)>,
    tags_manager: Res<GameplayTagsManager>,
    mut commands: Commands,
    keyboard: Res<ButtonInput<KeyCode>>,
) {
    let buff_tag = gameplay_tag!(tags::BUFF_STRENGTH);

    if keyboard.just_pressed(KeyCode::KeyB) {
        for (entity, name, mut tag_container) in query.iter_mut() {
            if name.as_str() == "Player" {
                info!("玩家获得增益!");
                tag_container.set_tag_count(&buff_tag, 3, &tags_manager, &mut commands, entity);
            }
        }
    }

    if keyboard.just_pressed(KeyCode::KeyR) {
        for (entity, name, mut tag_container) in query.iter_mut() {
            if name.as_str() == "Player" {
                let current_count = tag_container.get_tag_count(&buff_tag);
                if current_count > 0 {
                    info!("移除1层增益,当前: {}", current_count);
                    tag_container.update_tag_count(&buff_tag, -1, &tags_manager, &mut commands, entity);
                }
            }
        }
    }
}

fn display_tag_counts(
    query: Query<(&Name, &GameplayTagCountContainer)>,
    keyboard: Res<ButtonInput<KeyCode>>,
) {
    if keyboard.just_pressed(KeyCode::KeyI) {
        let damage_tag = gameplay_tag!(tags::DAMAGED);
        let buff_tag = gameplay_tag!(tags::BUFF_STRENGTH);

        info!("=== 标签计数信息 ===");
        for (name, tag_container) in query.iter() {
            info!("实体: {}", name);
            info!("  受伤层数: {}", tag_container.get_tag_count(&damage_tag));
            info!("  增益层数: {}", tag_container.get_tag_count(&buff_tag));
            if tag_container.has_matching_gameplay_tag(&damage_tag) {
                info!("  当前处于受伤状态");
            }
        }
    }
}
