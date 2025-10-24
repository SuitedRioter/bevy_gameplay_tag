use std::time::Duration;

use bevy::{prelude::*, time::common_conditions::on_timer};
use bevy_gameplay_tag::{
    gameplay_tag::GameplayTag, gameplay_tag_container::GameplayTagContainer,
    gameplay_tags_manager::GameplayTagsManager, gameplay_tags_plugin::GameplayTagsPlugin,
};
use bevy_inspector_egui::{bevy_egui::EguiPlugin, quick::WorldInspectorPlugin};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(EguiPlugin::default())
        .add_plugins(WorldInspectorPlugin::new())
        .add_plugins(GameplayTagsPlugin)
        .add_systems(Startup, setup)
        .add_systems(
            Update,
            test_gameplay_tag_match.run_if(on_timer(Duration::from_millis(200))),
        )
        .run();
}

fn setup(mut commands: Commands) {
    // camera
    commands.spawn((
        Name::new("Camera"),
        Camera3d::default(),
        Transform::from_xyz(-2.0, 2.5, 5.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));
}

fn test_gameplay_tag_match(tags_manager: Res<GameplayTagsManager>, world: &World) {
    let tag_a_b_c = GameplayTag::new("A.B.C");
    let mut a_tag_container = GameplayTagContainer::new();
    a_tag_container.add_tag(tag_a_b_c.clone(), &tags_manager, world);

    let tag_a_b = GameplayTag::new("A.B");
    let tag_d_c_b = GameplayTag::new("D.C.B");
    let mut b_tag_container = GameplayTagContainer::new();
    b_tag_container.add_tag(tag_a_b.clone(), &tags_manager, world);
    b_tag_container.add_tag(tag_d_c_b.clone(), &tags_manager, world);

    let mut other_container = GameplayTagContainer::new();
    other_container.add_tag(tag_d_c_b, &tags_manager, world);

    let has_tag = a_tag_container.has_tag(&tag_a_b_c); //true
    let has_tag_exact = a_tag_container.has_tag_exact(&tag_a_b); //false
    let has_any = a_tag_container.has_any(&b_tag_container); //true
    let has_any_2 = b_tag_container.has_any(&other_container); //true
    let has_any_exact = b_tag_container.has_any_exact(&other_container); //true
    let has_all = other_container.has_all(&b_tag_container); //false
    b_tag_container.remove_tag(&tag_a_b, false, &tags_manager, world);
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
