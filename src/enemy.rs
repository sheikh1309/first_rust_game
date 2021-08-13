use std::{f32::consts::PI};

use bevy::{core::{FixedTimestep}, prelude::*};
use rand::{Rng, thread_rng};

use crate::{Materials, SCALE, Speed, TIME_PER_FRAME, WindowSize, player::Laser};

const MAX_ENEMIES: u32 = 5;
const MAX_FORMATION_MEMBERS: u32 = 2;
pub struct ActiveEnemies(pub u32);

pub struct Enemy;
pub struct FromEnemy;
pub struct EnemyPlugin;

#[derive(Default, Clone)]
struct Formation {
    start: (f32, f32),
    radius: (f32, f32),
    offset: (f32, f32),
    angle: f32,
    group_id: u32
}

#[derive(Default)]
struct FormationMaker {
    group_seq: u32,
    current_formation: Option<Formation>,
    current_formation_members: u32
}

impl FormationMaker {
    fn make(&mut self, window_size: &WindowSize) -> Formation {
        match (&self.current_formation, self.current_formation_members >= MAX_FORMATION_MEMBERS) {
            // if first formation or previous formation null
            (None, _) | (_, true) => {
                // compute the start x/y
                let mut rng = thread_rng();
                let (h_span, w_span) = (window_size.height / 2. - 100., window_size.width / 4.);
                let x = if rng.gen::<bool>() { window_size.width } else { window_size.height };
                let y = rng.gen_range(-h_span..h_span) as f32;
                let start = (x, y);

                // compute offset and radius
                let offset = (rng.gen_range(-w_span..w_span), rng.gen_range(0.0..h_span));
                let radius = (rng.gen_range(80.0..150.0), 100.);
                let angle: f32 = (y - offset.0).atan2(x - offset.1);

                // create new formation
                self.group_seq += 1;
                let group_id = self.group_seq;
                let formation = Formation { start, offset, radius, angle, group_id };
                self.current_formation = Some(formation.clone());
                self.current_formation_members = 1;
                formation
            }
            // if still within the formation count
            (Some(formation), false) => {
                self.current_formation_members += 1;
                formation.clone()
            }
        }
    }
}


impl Plugin for EnemyPlugin {
    fn build(&self, app: &mut AppBuilder) {
       app
           .insert_resource(FormationMaker::default())
           .add_system(enemy_laser_movment.system())
           .add_system(enemy_movment.system())
           .add_system_set(
                    SystemSet::new()
                    .with_run_criteria(FixedTimestep::step(1.0))
                    .with_system(enemy_spawn.system())
            ).add_system_set(
                SystemSet::new()
                .with_run_criteria(FixedTimestep::step(0.9))
                .with_system(enemy_fire.system())
            );
    }
}


fn enemy_spawn(
    mut commands: Commands,
    mut active_enemies: ResMut<ActiveEnemies>,
    mut formation_maker: ResMut<FormationMaker>,
    materials: Res<Materials>,
    window_size: Res<WindowSize>
) {
    if active_enemies.0 < MAX_ENEMIES {
        let formation = formation_maker.make(&window_size);
        let (x, y) = formation.start;
        commands.spawn_bundle(SpriteBundle {
            material: materials.enemy.clone(),
            transform: Transform {
                translation: Vec3::new(x, y, 10.),
                scale: Vec3::new(SCALE, SCALE, 1.),
                ..Default::default()
            },
            ..Default::default()
        })
        .insert(Speed::default())
        .insert(Enemy)
        .insert(formation);

        active_enemies.0 += 1;
    }
}


fn enemy_fire(
    mut commands: Commands,
    materials: Res<Materials>,
    enemy_quert: Query<&Transform, With<Enemy>>
) {
   for &tf in enemy_quert.iter() {
        let (x, y) = (tf.translation.x, tf.translation.y);
        commands
            .spawn_bundle(
                SpriteBundle {
                    material: materials.enemy_laser.clone(),
                    transform: Transform {
                        translation: Vec3::new(x, y - 15., 0.),
                        scale: Vec3::new(SCALE, -SCALE, 1.),
                        ..Default::default()
                    },
                    ..Default::default()
                }
            )
            .insert(Laser)
            .insert(FromEnemy)
            .insert(Speed::default());
   } 
}

fn enemy_laser_movment(
    mut commands: Commands,
    window_size: Res<WindowSize>,
    mut laser_query: Query<(Entity, &Speed, &mut Transform), (With<Laser>, With<FromEnemy>)>
) {
    for (entity, speed, mut tf) in laser_query.iter_mut() {
        tf.translation.y -= speed.0 * TIME_PER_FRAME;
        if tf.translation.y < -window_size.height / 2. - 50. {
            commands.entity(entity).despawn();
        }
    }
}


fn enemy_movment(mut query: Query<(&mut Transform, &Speed, &mut Formation), With<Enemy>>) {
    for (mut tf, speed, mut formation) in query.iter_mut() {
        let max_distance = TIME_PER_FRAME * speed.0;
        let (x_org, y_org) = (tf.translation.x, tf.translation.y);
        
        // Get the ellipse
        let (x_offset, y_offset) = formation.offset;
        let (x_radius, y_radius) = formation.radius;

        // Compute the destination
        let dir = if formation.start.0 > 0. { 1. } else { -1. };
        let angle = formation.angle + dir * speed.0 * TIME_PER_FRAME / (x_radius.min(y_radius) * PI / 2.);
            
        // Calculate the destination
        let x_dst = x_radius * angle.cos() + x_offset;
        let y_dst = y_radius * angle.sin() + y_offset;

        // Calculate the distance
        let (delta_x, delta_y) = (x_org - x_dst, y_org - y_dst);
        let distance = (delta_x * delta_x + delta_y * delta_y).sqrt();

        let distance_ratio = if distance == 0. { 0. } else { max_distance / distance };
        
        // Calculate the final x/y (make sure to not overshoot)
        let x = x_org - delta_x * distance_ratio;
        let y = y_org - delta_y * distance_ratio;

        if distance < max_distance * speed.0 / 20. {
            formation.angle = angle;
        }

        tf.translation.x = if delta_x > 0. { x.max(x_dst) } else { x.min(x_dst) };
        tf.translation.y = if delta_y > 0. { y.max(y_dst) } else { y.min(y_dst) };
    }

}


