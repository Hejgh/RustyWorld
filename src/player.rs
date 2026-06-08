use bevy::prelude::*;

#[derive(Component)]
pub struct Player;

pub fn spawn_player(mut commands: Commands) {
    commands.spawn((
        SpriteBundle {
            sprite: Sprite { color: Color::srgb(1.0, 0.0, 0.0), custom_size: Some(Vec2::new(30.0, 30.0)), ..default() },
            ..default()
        },
        Player,
    ));
}
