use std::time::Duration;
use bevy::prelude::*;
use bevy::time::common_conditions::on_timer;
use bevy_gameplay_tag::{
    gameplay_tag::GameplayTag,
    gameplay_tag_container::GameplayTagContainer,
    gameplay_tag_count_container::{
        GameplayTagCountContainer, GameplayTagEventType, OnGameplayEffectTagCountChanged,
    },
    gameplay_tags_manager::GameplayTagsManager,
    gameplay_tags_plugin::GameplayTagsPlugin,
};
use bevy_inspector_egui::{bevy_egui::EguiPlugin, quick::WorldInspectorPlugin};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(EguiPlugin::default())
        .add_plugins(WorldInspectorPlugin::new())
        //.add_plugins(GameplayTagsPlugin::new())
        .add_plugins(GameplayTagsPlugin::with_data_path("examples/tag_data.json".to_string()))
        .add_systems(Startup, setup)
        .add_systems(
            Update,
            (
                test_gameplay_tag_match.run_if(on_timer(Duration::from_millis(200))),
                //apply_damage_system,
                //apply_buff_system,
                //display_tag_counts,
            ),
        )
        .run();
}

fn setup(mut commands: Commands) {
    // Player
    let player = commands
        .spawn((Name::new("Player"), GameplayTagCountContainer::new()))
        .id();

    // 为玩家添加观察者,监听标签计数变化
    commands.entity(player).observe(on_player_tag_changed);

    // Enemy
    let enemy = commands
        .spawn((Name::new("Enemy"), GameplayTagCountContainer::new()))
        .id();

    commands.entity(enemy).observe(on_enemy_tag_changed);

    // camera
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
            info!(
                "{} 标签 {:?} 计数变更为: {}",
                name, event.tag, event.new_count
            );
        }
    }
}

fn on_enemy_tag_changed(trigger: On<OnGameplayEffectTagCountChanged>, query: Query<&Name>) {
    let event = trigger.event();
    let name = query.get(event.entity).unwrap();

    // 只关注重大变化(新增或移除)
    if event.event_type == GameplayTagEventType::NewOrRemoved {
        info!(
            "敌人 {} 标签状态变化: {:?} -> {}",
            name, event.tag, event.new_count
        );
    }
}

fn apply_damage_system(
    mut query: Query<(Entity, &Name, &mut GameplayTagCountContainer)>,
    tags_manager: Res<GameplayTagsManager>,
    mut commands: Commands,
    keyboard: Res<ButtonInput<KeyCode>>,
) {
    if keyboard.just_pressed(KeyCode::Space) {
        for (entity, name, mut tag_container) in query.iter_mut() {
            if name.as_str() == "Player" {
                info!("玩家受到伤害!");
                let damage_tag = GameplayTag::new("Status.Damaged");
                tag_container.update_tag_count(
                    &damage_tag,
                    1,
                    &tags_manager,
                    &mut commands,
                    entity,
                );
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
    if keyboard.just_pressed(KeyCode::KeyB) {
        for (entity, name, mut tag_container) in query.iter_mut() {
            if name.as_str() == "Player" {
                info!("玩家获得增益!");
                // 设置增益标签计数为3
                let buff_tag = GameplayTag::new("Buff.Strength");
                tag_container.set_tag_count(
                    &buff_tag,
                    3, // 直接设置为3层
                    &tags_manager,
                    &mut commands,
                    entity,
                );
            }
        }
    }

    if keyboard.just_pressed(KeyCode::KeyR) {
        for (entity, name, mut tag_container) in query.iter_mut() {
            if name.as_str() == "Player" {
                let buff_tag = GameplayTag::new("Buff.Strength");
                let current_count = tag_container.get_tag_count(&buff_tag);
                if current_count > 0 {
                    info!("移除1层增益,当前: {}", current_count);
                    // 减少1层
                    tag_container.update_tag_count(
                        &buff_tag,
                        -1,
                        &tags_manager,
                        &mut commands,
                        entity,
                    );
                };
            }
        }
    }
}

fn display_tag_counts(
    query: Query<(&Name, &GameplayTagCountContainer)>,
    keyboard: Res<ButtonInput<KeyCode>>,
) {
    if keyboard.just_pressed(KeyCode::KeyI) {
        info!("=== 标签计数信息 ===");
        for (name, tag_container) in query.iter() {
            info!("实体: {}", name);
            // 检查特定标签
            let damage_tag = GameplayTag::new("Status.Damaged");
            let buff_tag = GameplayTag::new("Buff.Strength");

            let damage_count = tag_container.get_tag_count(&damage_tag);
            let buff_count = tag_container.get_tag_count(&buff_tag);

            info!("  受伤层数: {}", damage_count);
            info!("  增益层数: {}", buff_count);

            // 检查是否有匹配的标签
            if tag_container.has_matching_gameplay_tag(&damage_tag) {
                info!("  当前处于受伤状态");
            }
        }
    }
}

#[allow(dead_code)]
fn test_gameplay_tag_match(tags_manager: Res<GameplayTagsManager>) {
    let tag_a_b_c = GameplayTag::new("Cooldown.Skill.S13");
    let mut a_tag_container = GameplayTagContainer::new();
    a_tag_container.add_tag(tag_a_b_c.clone(), &tags_manager);

    let tag_a_b = GameplayTag::new("Cooldown.Skill");
    let tag_d_c_b = GameplayTag::new("Item.Type.SkillFragment");
    let mut b_tag_container = GameplayTagContainer::new();
    b_tag_container.add_tag(tag_a_b.clone(), &tags_manager);
    b_tag_container.add_tag(tag_d_c_b.clone(), &tags_manager);

    let mut other_container = GameplayTagContainer::new();
    other_container.add_tag(tag_d_c_b, &tags_manager);

    let has_tag = a_tag_container.has_tag(&tag_a_b_c); //true
    let has_tag_exact = a_tag_container.has_tag_exact(&tag_a_b); //false
    let has_any = a_tag_container.has_any(&b_tag_container); //true
    let has_any_2 = b_tag_container.has_any(&other_container); //true
    let has_any_exact = b_tag_container.has_any_exact(&other_container); //true
    let has_all = other_container.has_all(&b_tag_container); //false
    b_tag_container.remove_tag(&tag_a_b, false, &tags_manager);
    let has_all_2 = other_container.has_all(&b_tag_container); //true
    let has_all_exact = b_tag_container.has_all(&other_container); //true

    println!(
        "Debug: {:?}, {:?}, {:?}, {:?}, {:?}, {:?}, {:?}, {:?}",
        has_tag,
        has_tag_exact,
        has_any,
        has_any_2,
        has_any_exact,
        has_all,
        has_all_2,
        has_all_exact
    );
}
