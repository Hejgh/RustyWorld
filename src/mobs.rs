use bevy::prelude::*;
use rand::Rng;
use crate::world::{Player, WorldChunks};
use crate::time::WorldTime;

#[derive(Component)]
pub struct Mob {
    pub health: f32,
    pub max_health: f32,
    pub speed: f32,
    pub attack_damage: f32,
    pub attack_range: f32,
    pub state: MobState,
}

#[derive(Component)]
pub struct Bomber;

#[derive(Component)]
pub struct Devil;

#[derive(Component)]
pub struct Sniper;

#[derive(Component)]
pub struct DeathWish;

pub enum MobState {
    Idle { wander_target: Option<Vec3>, timer: Timer },
    Chase { target: Entity, last_seen: Vec3 },
    Attack { cooldown: Timer },
    Explode { fuse: Timer },
}

pub fn spawn_mobs(
    mut commands: Commands,
    time: Res<WorldTime>,
    player_query: Query<&Transform, With<Player>>,
    world_chunks: Res<WorldChunks>,
    existing_mobs: Query<&Mob>,
) {
    if !time.is_night() && !time.is_full_moon() {
        return;
    }
    
    let Ok(player_pos) = player_query.get_single() else { return };
    let mut rng = rand::thread_rng();
    
    fn has_boss(mobs: &Query<&Mob>) -> bool {
        mobs.iter().any(|mob| mob.max_health >= 500.0)
    }
    
    if time.is_full_moon() && !has_boss(&existing_mobs) && rng.gen_bool(0.28) {
        spawn_deathwish(&mut commands, player_pos.translation, &world_chunks);
        return;
    }
    
    if rng.gen_bool(0.05) {
        let mob_type: i32 = rng.gen_range(0..3);
        let pos = random_surface_position(player_pos.translation, &world_chunks);
        
        match mob_type {
            0 => spawn_bomber(&mut commands, pos),
            1 => spawn_devil(&mut commands, pos),
            2 => spawn_sniper(&mut commands, pos),
            _ => {}
        }
    }
}

fn spawn_bomber(commands: &mut Commands, pos: Vec3) {
    commands.spawn((
        Mob {
            health: 40.0,
            max_health: 40.0,
            speed: 1.5,
            attack_damage: 50.0,
            attack_range: 2.5,
            state: MobState::Idle { wander_target: None, timer: Timer::from_seconds(2.0, TimerMode::Repeating) },
        },
        Bomber,
        Transform::from_translation(pos),
    ));
}

fn spawn_devil(commands: &mut Commands, pos: Vec3) {
    commands.spawn((
        Mob {
            health: 60.0,
            max_health: 60.0,
            speed: 3.5,
            attack_damage: 15.0,
            attack_range: 2.0,
            state: MobState::Idle { wander_target: None, timer: Timer::from_seconds(1.0, TimerMode::Repeating) },
        },
        Devil,
        Transform::from_translation(pos),
    ));
}

fn spawn_sniper(commands: &mut Commands, pos: Vec3) {
    commands.spawn((
        Mob {
            health: 30.0,
            max_health: 30.0,
            speed: 1.2,
            attack_damage: 25.0,
            attack_range: 32.0,
            state: MobState::Idle { wander_target: None, timer: Timer::from_seconds(3.0, TimerMode::Repeating) },
        },
        Sniper,
        Transform::from_translation(pos),
    ));
}

fn spawn_deathwish(commands: &mut Commands, player_pos: Vec3, _world_chunks: &WorldChunks) {
    let spawn_pos = Vec3::new(player_pos.x + 10.0, 70.0, player_pos.z + 10.0);
    commands.spawn((
        Mob {
            health: 500.0,
            max_health: 500.0,
            speed: 2.0,
            attack_damage: 30.0,
            attack_range: 4.0,
            state: MobState::Idle { wander_target: None, timer: Timer::from_seconds(1.0, TimerMode::Repeating) },
        },
        DeathWish,
        Transform::from_translation(spawn_pos),
    ));
}

fn random_surface_position(around: Vec3, _world_chunks: &WorldChunks) -> Vec3 {
    let mut rng = rand::thread_rng();
    let x = around.x + rng.gen_range(-32.0..32.0);
    let z = around.z + rng.gen_range(-32.0..32.0);
    Vec3::new(x, 70.0, z)
}

pub fn update_mobs(
    mut mobs: Query<(Entity, &mut Mob, &mut Transform, Option<&Bomber>, Option<&DeathWish>)>,
    player_query: Query<&Transform, With<Player>>,
    time: Res<Time>,
    mut commands: Commands,
) {
    let Ok(player_transform) = player_query.get_single() else { return };
    let player_entity = player_query.iter().next().unwrap();
    
    for (entity, mut mob, mut transform, is_bomber, is_boss) in mobs.iter_mut() {
        let distance = transform.translation.distance(player_transform.translation);
        
        match &mut mob.state {
            MobState::Idle { wander_target, timer } => {
                timer.tick(time.delta());
                if timer.just_finished() {
                    let mut rng = rand::thread_rng();
                    *wander_target = Some(transform.translation + Vec3::new(
                        rng.gen_range(-10.0..10.0),
                        0.0,
                        rng.gen_range(-10.0..10.0),
                    ));
                }
                
                if let Some(target) = wander_target {
                    let direction = (*target - transform.translation).normalize_or_zero();
                    transform.translation += direction * mob.speed * time.delta_seconds();
                    
                    if transform.translation.distance(*target) < 0.5 {
                        *wander_target = None;
                    }
                }
                
                if distance < 32.0 {
                    mob.state = MobState::Chase { target: player_entity, last_seen: player_transform.translation };
                }
            }
            
            MobState::Chase { target, last_seen: _ } => {
                let direction = (player_transform.translation - transform.translation).normalize_or_zero();
                transform.translation += direction * mob.speed * time.delta_seconds();
                
                if distance < mob.attack_range {
                    mob.state = MobState::Attack { cooldown: Timer::from_seconds(1.0, TimerMode::Once) };
                } else if distance > 40.0 {
                    mob.state = MobState::Idle { wander_target: None, timer: Timer::from_seconds(2.0, TimerMode::Repeating) };
                }
            }
            
            MobState::Attack { cooldown } => {
                cooldown.tick(time.delta());
                
                if cooldown.just_finished() {
                    if is_bomber.is_some() {
                        mob.state = MobState::Explode { fuse: Timer::from_seconds(2.0, TimerMode::Once) };
                    } else {
                        println!("Mob attacks player for {} damage!", mob.attack_damage);
                        mob.state = MobState::Chase { target: player_entity, last_seen: player_transform.translation };
                    }
                }
            }
            
            MobState::Explode { fuse } => {
                fuse.tick(time.delta());
                if fuse.just_finished() {
                    println!("💥 BOMBER EXPLODES at {:?}", transform.translation);
                    commands.entity(entity).despawn();
                }
            }
        }
        
        if is_boss.is_some() && mob.health < mob.max_health * 0.5 {
            mob.speed = 2.5;
        }
        
        if mob.health <= 0.0 {
            commands.entity(entity).despawn();
        }
    }
}
