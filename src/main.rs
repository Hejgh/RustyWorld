use bevy::prelude::*;

mod world;
mod mobs;
mod time;

use world::{WorldChunks, manage_chunks, Player, ChunkHandle};
use mobs::{spawn_mobs, update_mobs};
use time::{WorldTime, update_day_night};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .insert_resource(WorldTime::new())
        .insert_resource(WorldChunks::new())
        .add_systems(Startup, setup)
        .add_systems(Update, (
            update_day_night,
            manage_chunks,
            spawn_mobs,
            update_mobs,
            player_movement,
        ).chain())
        .run();
}

fn setup(mut commands: Commands) {
    commands.spawn(Camera3dBundle {
        transform: Transform::from_xyz(0.0, 80.0, 80.0).looking_at(Vec3::ZERO, Vec3::Y),
        ..default()
    });
    
    commands.spawn((
        Player,
        Transform::from_xyz(0.0, 70.0, 0.0),
    ));
}

fn player_movement(
    mut player_query: Query<&mut Transform, With<Player>>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
) {
    let Ok(mut transform) = player_query.get_single_mut() else { return };
    let mut direction = Vec3::ZERO;
    
    if keyboard_input.pressed(KeyCode::KeyW) { direction.z -= 1.0; }
    if keyboard_input.pressed(KeyCode::KeyS) { direction.z += 1.0; }
    if keyboard_input.pressed(KeyCode::KeyA) { direction.x -= 1.0; }
    if keyboard_input.pressed(KeyCode::KeyD) { direction.x += 1.0; }
    
    if direction != Vec3::ZERO {
        direction = direction.normalize();
        transform.translation += direction * 10.0 * time.delta_seconds();
    }
}
