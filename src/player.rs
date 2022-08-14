use bevy::{
    input::mouse::MouseMotion,
    prelude::*
};
use bevy_rapier3d::prelude::{
    *,
    CollisionEvent::{
        Started,
        Stopped 
    }
};

use std::f32::consts::PI;
use std::collections::HashMap;
use std::collections::hash_map::Entry;

#[derive(Component)]
pub struct PlayerStatus {
    pitch: f32,
    grounds: HashMap<u32, bool>,
    on_ground: bool,
    jump_velocity: Vec3
}

impl Default for PlayerStatus {
    fn default() -> Self {
        Self{
            pitch: 0.0,
            grounds: HashMap::new(),
            on_ground: true,
            jump_velocity: Vec3::ZERO
        }
    }
}
impl PlayerStatus {   
    fn ground_remove(&mut self, index: u32) {
        if let Entry::Occupied(o) = self.grounds.entry(index.try_into().unwrap()) {
            o.remove_entry();
        }
    }
}

#[derive(Component)]
pub struct Player;

pub fn setup_player(
    mut commands: Commands,
    mut windows: ResMut<Windows>,
    asset_server: Res<AssetServer>,
) {
    let window = windows.get_primary_mut().unwrap();
    window.set_cursor_visibility(false);
    
    //player entity
    commands
        .spawn()
        .insert(Player)
        .insert(PlayerStatus::default())
        .insert_bundle(TransformBundle::from(Transform::from_xyz(0.0, 2.0, 0.0)))
        //Physical Body
        .insert(RigidBody::Dynamic)
        .insert(
            LockedAxes::ROTATION_LOCKED_X
            | LockedAxes::ROTATION_LOCKED_Z
        )
        .insert(Sleeping::disabled())
        .insert(Collider::cuboid(0.5, 1.0, 0.5))
        .with_children(|parent| {
            parent.spawn()
                .insert(Collider::cuboid(0.5, 0.05, 0.5))
                .insert(ActiveEvents::COLLISION_EVENTS)
                .insert(Sensor)
                .insert_bundle(TransformBundle::from(Transform::from_xyz(0.0, -0.96, 0.0)));

            //create a camera
            parent.spawn_bundle(Camera3dBundle {
                // when you want to see your self, change the coordinate of z
                transform: Transform::from_xyz(0.0, 0.5, 0.0),
                ..default()
            });
        });
    
        //create cursor
        let texture_handle = asset_server.load("cursor.png");
        commands.spawn_bundle(ImageBundle{
            image: UiImage(texture_handle.clone()),
            style: Style {
                align_self: AlignSelf::FlexEnd,
                position_type: PositionType::Absolute,
                position: UiRect {
                    top: Val::Percent(50.0),
                    left: Val::Percent(50.0),
                    ..default()
                },
                ..default()
            },
            ..default()
            }
        );
}

fn to_radians(x: f32) -> f32 { x * PI / 180.0 }

pub fn ground_event(
    mut collision_events: EventReader<CollisionEvent>,
    mut status: Query<&mut PlayerStatus>
) {
    let mut player_status = match status.get_single_mut() {
        Ok(status) => status,
        _ => {
            error!("Player Status not found.");
            return;
        }
    };use std::f32::consts::PI;
use std::collections::HashMap;
use std::collections::hash_map::Entry;
    
    for collision_event in collision_events.iter() {
        match collision_event {
            Started(_, ground, _) => {
                player_status.grounds.insert(ground.id(), true);
            },
            Stopped(_, ground, _) => {
                player_status.ground_remove(ground.id());
            }
        }
    }
    if player_status.grounds.keys().len() == 0 {
        player_status.on_ground = false;
    } else {
        player_status.on_ground = true;
    }
}

pub fn player_update(
    keyboard_input: Res<Input<KeyCode>>,
    mouse_motion: Res<Events<MouseMotion>>,
    time: Res<Time>,
    mut player_entity: Query<(&mut Transform, &mut PlayerStatus), (With<Player>, Without<Camera3d>)>,
    mut camera_transforms: Query<&mut Transform, (With<Camera3d>, Without<Player>)>,
) {
    const TURNOVER_RATE: f32 = 0.25;
    let (mut transform, mut status) = match player_entity.iter_mut().next() {
        Some((transform, status)) => (transform, status),
        _ => {
            error!("Player not found.");
            return;
        }
    };
    
    let mut camera_transform = match camera_transforms.get_single_mut() {
        Ok(camera_transform) => camera_transform,
        _ => {
            error!("Camera not found.");
            return;
        }
    };
    
    let look_vec = transform.forward();
    let side_vec = Vec3::new(look_vec.z, 0.0, -look_vec.x);
    let mut velocity = Vec3::new(0.0, 0.0, 0.0);

    //when you press W, S, A, D, the player will move
    if keyboard_input.pressed(KeyCode::W) {
        velocity += look_vec;
    }

    if keyboard_input.pressed(KeyCode::S) {
        velocity -= look_vec;
    }

    if keyboard_input.pressed(KeyCode::A) {
        velocity += side_vec;
    }

    if keyboard_input.pressed(KeyCode::D) {
        velocity -= side_vec;
    }

    if keyboard_input.pressed(KeyCode::Space) && status.on_ground {
        status.jump_velocity = Vec3::new(0.0, 3.0, 0.0)
    }
    if status.jump_velocity.to_array()[1] < 0.0 {
        status.jump_velocity = Vec3::ZERO;
    } else if status.jump_velocity != Vec3::ZERO {
        velocity += status.jump_velocity;
        status.jump_velocity -= Vec3::new(0.0, 0.15, 0.0);
    }

    if velocity != Vec3::ZERO {
        let dv = velocity * time.delta_seconds()*3.0;
        transform.translation += dv;
    }

    //event reader
    let mut reader = mouse_motion.get_reader();
    let mut delta = Vec2::ZERO;
    for event in reader.iter(&mouse_motion) {
        delta += event.delta;
    }

    let yaw = Quat::from_rotation_y(to_radians(-delta.to_array()[0]*TURNOVER_RATE));
    let previous_pitch = status.pitch;
    status.pitch = (status.pitch - delta.to_array()[1]*TURNOVER_RATE).clamp(-90.0, 90.0);
    let pitch = Quat::from_rotation_x(to_radians(status.pitch - previous_pitch));
    transform.rotation = yaw * transform.rotation;
    camera_transform.rotation = camera_transform.rotation * pitch;
}

pub fn player_sight_line(
    rapier_context: Res<RapierContext>,
    player: Query<Entity, With<Player>>,
    camera: Query<&GlobalTransform, With<Camera3d>>
) {
    let player_handle
        = match player.get_single() {
        Ok(entity) => entity,
        _ => {
            error!("Player not found.");
            return;
        }
    };
    let transform
        = match camera.get_single() {
        Ok(transform) => transform,
        _ => {
            error!("Player not found.");
            return;
        }
    };
    
    let ray_ori = transform.translation();
    let ray_dir = transform.forward();
    let max_toi = 5.0;
    let solid = true;
    let filter = QueryFilter::new().exclude_rigid_body(player_handle);
    
    if let Some((entity, toi)) = rapier_context.cast_ray(
        ray_ori, ray_dir, max_toi, solid, filter
    ) {
        // The first collider hit has the entity `entity` and it hit after
        // the ray travelled a distance equal to `ray_dir * toi`.
        let hit_point = ray_ori + ray_dir * toi;
        println!("Entity {:?} hit at point {}", entity, hit_point);
    } else {
        println!("===========================");
    }
}