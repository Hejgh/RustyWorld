use bevy::prelude::*;

pub fn generate_terrain(mut commands: Commands) {
    // Eventually, put your FastNoiseLite logic here
    for x in -10..10 {
        commands.spawn(SpriteBundle {
            sprite: Sprite { color: Color::srgb(0.0, 0.8, 0.0), custom_size: Some(Vec2::new(30.0, 30.0)), ..default() },
            transform: Transform::from_xyz(x as f32 * 32.0, -100.0, 0.0),
            ..default()
        });
    }
}
