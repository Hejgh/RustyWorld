use bevy::prelude::*;

// 1. Define your components
#[derive(Component)]
struct Player;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_systems(Startup, (setup_camera, spawn_player, generate_terrain))
        .run();
}

fn setup_camera(mut commands: Commands) {
    commands.spawn(Camera2dBundle::default());
}

fn spawn_player(mut commands: Commands) {
    commands.spawn((
        SpriteBundle {
            sprite: Sprite { color: Color::srgb(1.0, 0.0, 0.0), custom_size: Some(Vec2::new(30.0, 30.0)), ..default() },
            ..default()
        },
        Player, // The component that makes this our player
    ));
}

fn generate_terrain(mut commands: Commands) {
    // This is where you call your FastNoiseLite logic
    // For now, let's just spawn a single "chunk" of tiles
    for x in -10..10 {
        let height = 5.0; // This would be your noise.get_noise(x) * scale
        commands.spawn(SpriteBundle {
            sprite: Sprite { color: Color::srgb(0.0, 0.8, 0.0), custom_size: Some(Vec2::new(30.0, 30.0)), ..default() },
            transform: Transform::from_xyz(x as f32 * 32.0, height * -10.0, 0.0),
            ..default()
        });
    }
}
