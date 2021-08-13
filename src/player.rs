use bevy::{core::FixedTimestep, prelude::*};

use crate::{Materials, SCALE, Speed, TIME_PER_FRAME, WindowSize};

const PLAYER_SPRITE_HEIGHT: f32 = 75.;
const PLAYER_SPRITE_WIDTH: f32 = 144.;
const PLAYER_RESPAWN_DELAY: f64 = 2.;

pub struct Player;
pub struct Laser;
pub struct FromPlayer;
struct PlayerReadyFire(bool);
pub struct PlayerPlugin;
pub struct PlayerStatte {
    on: bool,
    last_shot: f64
}

impl Default for PlayerStatte {
    fn default() -> Self {
        Self {
            on: false,
            last_shot: 0.
        }   
    }
}

impl PlayerStatte {
    pub fn shot(&mut self, time: f64) {
        self.on = false;
        self.last_shot = time;
    }

    pub fn spawned(&mut self) {
        self.on = true;
        self.last_shot = 0.;
    }
}


impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut AppBuilder) {
       app
           .insert_resource(PlayerStatte::default())
           .add_startup_stage("game_setup_actors", SystemStage::single(player_spawn.system()))
           .add_system(player_movment.system())
           .add_system(player_fire.system())
           .add_system(laser_movment.system())
           .add_system_set(
               SystemSet::new()
               .with_run_criteria(FixedTimestep::step(0.5))
               .with_system(player_spawn.system())
            );
   
    }
}

fn player_spawn(
    mut commands: Commands,
    materials: Res<Materials>,
    window_size: Res<WindowSize>,
    time: Res<Time>,
    mut player_state: ResMut<PlayerStatte>
) {
    let now = time.seconds_since_startup();
    let last_shot = player_state.last_shot;
    let window_bottom_point = -window_size.height / 2.;
    let padding = 5.;
    
    if !player_state.on && (last_shot == 0. || now > last_shot + PLAYER_RESPAWN_DELAY) {
        commands.spawn_bundle(SpriteBundle {
            material: materials.player.clone(),
            transform: Transform {
                translation: Vec3::new(0., window_bottom_point + PLAYER_SPRITE_HEIGHT / 4. + padding, 10.),
                scale: Vec3::new(SCALE, SCALE, 1.0),
                ..Default::default()
            },
            ..Default::default()
        })
        .insert(Player)
        .insert(Speed::default())
        .insert(PlayerReadyFire(true))
        .insert(WindowSize { width: window_size.width, height: window_size.height });
        player_state.spawned();
    }
}


fn player_movment(
    keyboard_input: Res<Input<KeyCode>>,
    mut query: Query<(&Speed, &mut Transform, With<Player>, &WindowSize)>
) {
    if let Ok((speed, mut transform, _, window_size)) = query.single_mut() {
        let dir = if keyboard_input.pressed(KeyCode::Left) {
            -1.
        } else if keyboard_input.pressed(KeyCode::Right) {
            1.
        } else {
            0.
        };
        
        let movement = dir * speed.0 * TIME_PER_FRAME;
        let limit = (window_size.width / 2.) - (PLAYER_SPRITE_WIDTH / 4.);
        let reach_limit = transform.translation.x + movement > limit || transform.translation.x + movement < -limit;
        if reach_limit == false {
            transform.translation.x += movement;
        }
    }
}

fn player_fire(
    mut commands: Commands,
    materials: Res<Materials>,
    keyboard_input: Res<Input<KeyCode>>,
    mut query: Query<(&Transform, &mut PlayerReadyFire, With<Player>)>
) {
    if let Ok((transform, mut ready_fire, _)) = query.single_mut() {
        if ready_fire.0 && keyboard_input.pressed(KeyCode::Space) {
            let (x, y): (f32, f32) = (transform.translation.x, transform.translation.y);
            let mut spawn_lasers = |x_offset: f32| { 
                commands.spawn_bundle(SpriteBundle {
                    material: materials.player_laser.clone(),
                    transform: Transform { 
                        translation: Vec3::new(x + x_offset, y + 15., 0.),
                        ..Default::default()
                    },
                    ..Default::default() 
                })
                .insert(Laser)
                .insert(FromPlayer)
                .insert(Speed::default());
            };

            let x_offset = PLAYER_SPRITE_WIDTH / 4. - 5.;
            spawn_lasers(x_offset);
            spawn_lasers(-x_offset);
            
            ready_fire.0 = false;
        }

        if keyboard_input.just_released(KeyCode::Space) {
            ready_fire.0 = true;
        }
    }
}

fn laser_movment(
    mut commands: Commands,
    window_size: Res<WindowSize>,
    mut query: Query<(Entity, &Speed, &mut Transform, (With<Laser>, With<FromPlayer>))>
) {
    for (laser_entity, speed, mut transform, _) in query.iter_mut() {
        transform.translation.y += speed.0 * TIME_PER_FRAME;
        if transform.translation.y > window_size.height {
            commands.entity(laser_entity).despawn();
        }
    }
}
