use std::collections::HashSet;

use bevy::{prelude::*, sprite::collide_aabb::collide};
use enemy::{ActiveEnemies, Enemy, EnemyPlugin, FromEnemy};
use player::{FromPlayer, Laser, Player, PlayerPlugin, PlayerStatte};

mod player;
mod enemy;

const PLAYER_SPRITE: &str = "player_a_01.png";
const PLAYER_LASER_SPRITE: &str = "laser_a_01.png";
const ENEMY_LASER_SPRITE: &str = "laser_b_01.png";
const ENEMY_SPRITE: &str = "enemy_a_01.png";
const EXPLOSION_SHEET: &str = "explo_a_sheet.png";
const TIME_PER_FRAME: f32 = 1. / 60.;
const SCALE: f32 = 0.5;

pub struct Speed(f32);
impl Default for Speed {
    fn default() -> Self {
        Self (500.)
    }
}

pub struct Materials {
    player: Handle<ColorMaterial>,
    player_laser: Handle<ColorMaterial>,
    enemy_laser: Handle<ColorMaterial>,
    enemy: Handle<ColorMaterial>,
    explosion: Handle<TextureAtlas>
}

pub struct WindowSize {
    width: f32,
    height: f32
}

struct Explosion;
struct ExplosionToSpawn(Vec3);

fn main() {
    let window_descriptor = WindowDescriptor {
        title: "Rust Game".to_string(),
        width: 600.0,
        height: 680.0,
        ..Default::default()
    };

    App::build()
        .insert_resource(ClearColor(Color::rgb(0.04, 0.04, 0.04)))
        .insert_resource(ActiveEnemies(0))
        .insert_resource(window_descriptor)
        .add_plugins(DefaultPlugins)
        .add_plugin(PlayerPlugin)
        .add_plugin(EnemyPlugin)
        .add_startup_system(setup.system())
        .add_system(player_laser_hit_enemy.system())
        .add_system(enemy_laser_hit_player.system())
        .add_system(explosion_to_spawn.system())
        .add_system(animate_explosion.system())
        .run();
}


fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut texture_atlases: ResMut<Assets<TextureAtlas>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut windows: ResMut<Windows>
) {
    let window = windows.get_primary_mut().unwrap();
    // camera
    commands.spawn_bundle(OrthographicCameraBundle::new_2d());
    
    let texture_handle = asset_server.load(EXPLOSION_SHEET);
    let texture_atlas = TextureAtlas::from_grid(texture_handle, Vec2::new(64.0, 64.0), 4, 4);

    commands.insert_resource(Materials {
        player: materials.add(asset_server.load(PLAYER_SPRITE).into()),
        player_laser: materials.add(asset_server.load(PLAYER_LASER_SPRITE).into()),
        enemy_laser: materials.add(asset_server.load(ENEMY_LASER_SPRITE).into()),
        enemy: materials.add(asset_server.load(ENEMY_SPRITE).into()),
        explosion: texture_atlases.add(texture_atlas)
    });
    commands.insert_resource(WindowSize {
        width: window.width(),
        height: window.height()
    });
}


fn player_laser_hit_enemy(
    mut commands: Commands,
    mut laser_query: Query<(Entity, &Transform, &Sprite, (With<Laser>, With<FromPlayer>))>,
    mut enemy_query: Query<(Entity, &Transform, &Sprite, With<Enemy>)>,
    mut active_enemies: ResMut<ActiveEnemies>
) {
    let mut enemies_blasted: HashSet<Entity> = HashSet::new();
    for (laser_entity, laser_tf, laser_sprite, _) in laser_query.iter_mut() {
        for (enemy_entity, enemy_tf, enemy_sprite, _) in enemy_query.iter_mut() {
            let laser_scale = Vec2::from(laser_tf.scale);
            let enemy_scale = Vec2::from(enemy_tf.scale);

            let collision = collide(
                laser_tf.translation,
                laser_sprite.size * laser_scale,
                enemy_tf.translation,
                enemy_sprite.size * enemy_scale
            );
            
            if let Some(_) = collision {
                if enemies_blasted.get(&enemy_entity).is_none() {
                   // remove the enemy
                    commands.entity(enemy_entity).despawn();
                    active_enemies.0 -= 1;
                    // spawn explosion to spawn
                    commands
                        .spawn()
                        .insert(ExplosionToSpawn(enemy_tf.translation.clone()));
                    
                    enemies_blasted.insert(enemy_entity);
                }
                
                // remove the laser
                commands.entity(laser_entity).despawn();
            }
        }
    }
}

fn enemy_laser_hit_player(
    mut commands: Commands,
    mut player_state: ResMut<PlayerStatte>,
    time: Res<Time>,
    laser_query: Query<(Entity, &Transform, &Sprite), (With<Laser>, With<FromEnemy>)>,
    player_query: Query<(Entity, &Transform, &Sprite), With<Player>>
) {
    if let Ok((player_entity, player_tf, player_sprite)) = player_query.single() {
        let player_size = player_sprite.size * Vec2::from(player_tf.scale.abs());
        for (laser_entity, laser_tf, laser_sprite) in laser_query.iter() {
            let laser_size = laser_sprite.size * Vec2::from(laser_tf.scale.abs());
            let collision = collide(laser_tf.translation, laser_size, player_tf.translation, player_size);
            if let Some(_) = collision {
                // remove the player from the view
                commands.entity(player_entity).despawn();
                player_state.shot(time.seconds_since_startup());
                //remove the laser
                commands.entity(laser_entity).despawn();
                // show ti fire
                commands.spawn().insert(ExplosionToSpawn(player_tf.translation.clone()));
            }
        }
    }
}

fn explosion_to_spawn(
    mut commands: Commands,
    query: Query<(Entity, &ExplosionToSpawn)>,
    materials: Res<Materials>
) {
    for (explosion_entity, explosion_to_spawn) in query.iter() {
        commands
            .spawn_bundle(SpriteSheetBundle {
                texture_atlas: materials.explosion.clone(),
                transform: Transform {
                    translation: explosion_to_spawn.0,
                    ..Default::default()
                },
                ..Default::default()
            })
            .insert(Explosion)
            .insert(Timer::from_seconds(0.05, true));

        commands.entity(explosion_entity).despawn();
    }
}

fn animate_explosion(
    mut commands: Commands,
    time: Res<Time>,
    texture_atlases: Res<Assets<TextureAtlas>>,
    mut query: Query<(
        Entity,
        &mut Timer,
        &mut TextureAtlasSprite,
        &Handle<TextureAtlas>,
        With<Explosion>
    )>
) {
    for (entity, mut timer, mut sprite, texture_atlas_handle, _) in query.iter_mut() {
        timer.tick(time.delta());
        if timer.finished() {
            let texture_atlas = texture_atlases.get(texture_atlas_handle).unwrap();
            sprite.index += 1;
            if sprite.index == texture_atlas.textures.len() as u32 {
                commands.entity(entity).despawn();
            }
        }
    }
}
