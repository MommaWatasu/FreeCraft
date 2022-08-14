use bevy::{
    ecs::event::Events,
    input::mouse::MouseMotion,
    prelude::*,
    render::texture::ImageSettings,
    ui::entity::ImageBundle
};
use bevy_rapier3d::prelude::{
    *,
    CollisionEvent::{
        Started,
        Stopped 
    }
};

mod block;
mod debugger;
mod player;
mod sky;

use block::Block;
use debugger::{
    Debugger, update_debugger
};
use player::{
    setup_player,
    ground_event,
    player_update,
    player_sight_line
};
use sky::{
    AtmospherePlugin,
    material::Atmosphere,
};

/// This example shows various ways to configure texture materials in 3D
fn main() {
    App::new()
        .insert_resource(Msaa { samples: 4 })
        .insert_resource(Atmosphere::default())
        .insert_resource(ImageSettings::default_nearest())
        .insert_resource(Debugger::default())
        .add_plugins(DefaultPlugins)
        .add_plugin(RapierPhysicsPlugin::<NoUserData>::default())
        .add_plugin(RapierDebugRenderPlugin::default()) //collision debugging
        .add_plugin(AtmospherePlugin::default())
        //.add_plugin(MaterialPlugin::<CubemapMaterial>::default())
        .add_startup_system(setup_player)
        .add_startup_system(setup_environment)
        .add_startup_system(terrain_generation)
        //.add_startup_system(load_skybox)
        //.add_system(create_skybox)
        .add_system(ground_event)
        .add_system(player_update)
        .add_system(update_debugger)
        .add_system(daylight_cycle)
        .add_system(player_sight_line)
        .run();
}

//player control starts at here
/*
#[derive(Component)]
struct PlayerStatus {
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

fn setup_player(
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

fn ground_event(
    mut collision_events: EventReader<CollisionEvent>,
    mut status: Query<&mut PlayerStatus>
) {
    let mut player_status = match status.get_single_mut() {
        Ok(status) => status,
        _ => {
            error!("Player Status not found.");
            return;
        }
    };
    
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

fn player_update(
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
*/
//player control ends at here

// the component for identify sun and moon
#[derive(Component)]
struct SunOrMoon {
    is_sun: bool
}

// We can edit the SkyMaterial resource and it will be updated automatically, as long as AtmospherePlugin.dynamic is true
fn daylight_cycle(
    mut sky_mat: ResMut<Atmosphere>,
    mut query: Query<(&mut Transform, &mut DirectionalLight, &SunOrMoon)>,
    time: Res<Time>,
) {
    let mut pos = sky_mat.sun_position;
    let t = time.time_since_startup().as_millis() as f32 / 200000.0;
    pos.y = t.sin();
    pos.z = t.cos();
    sky_mat.sun_position = pos;

    for (mut light_trans, mut directional, sun_type) in &mut query {
        if sun_type.is_sun {
            light_trans.rotation = Quat::from_rotation_x(-pos.y.atan2(pos.z));
            directional.illuminance = t.sin().max(0.0).powf(2.0) * 100000.0;
        } else {
            light_trans.rotation = Quat::from_rotation_x(pos.y.atan2(pos.z));
            directional.illuminance = t.sin().max(0.0).powf(2.0) * 1000.0;
        }
    }
    /*
    if let Some((mut light_trans, mut directional)) = query.single_mut().into() {
        light_trans.rotation = Quat::from_rotation_x(-pos.y.atan2(pos.z));
        directional.illuminance = t.sin().max(0.0).powf(2.0) * 100000.0;
    }
    */
}

fn setup_environment(
    mut commands: Commands,
) {
    // Our Sun
    commands
        .spawn_bundle(DirectionalLightBundle {
            ..Default::default()
        })
        .insert(SunOrMoon{ is_sun: true }); // Marks the light as Sun
    
    // Our Moon
    commands
        .spawn_bundle(DirectionalLightBundle {
            ..Default::default()
        })
        .insert(SunOrMoon{ is_sun: false }); // Marks the light as Moon
}

fn terrain_generation(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    for i in -3..3 {
        for j in -3..3 {
            create_block(&mut commands, &asset_server, &mut meshes, &mut materials, Vec3::new(i as f32, 0.0, j as f32))
        }
    }
    for i in -3..3 {
        for j in 3..6 {
            create_block(&mut commands, &asset_server, &mut meshes, &mut materials, Vec3::new(i as f32, -1.0, j as f32))
        }
    }
    
}

fn create_block(
    commands: &mut Commands,
    asset_server: &Res<AssetServer>,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    coord: Vec3
) {
    let (x, y, z) = (coord.x, coord.y, coord.z);
    let mut textures: [u32; 6] = [0; 6];
    let mut transform: Transform;
    let mut material: Handle<StandardMaterial>;
    let binding = asset_server.load("textures/stone.png");
    let texture_handles = [&binding; 6];
    let plane = meshes.add(Mesh::from(shape::Plane{ size: 1.0 }));

    textures[0] = {
        transform = Transform {
            translation: Vec3::new(x+0.5, y, z),
            rotation: Quat::from_rotation_x(-std::f32::consts::PI / 2.0),
            ..default()
        };
        transform.rotate(Quat::from_axis_angle(Vec3::Y, -std::f32::consts::PI / 2.0));

        material = materials.add(StandardMaterial {
            base_color_texture: Some(texture_handles[0].clone()),
            alpha_mode: AlphaMode::Blend,
            unlit: true,
            ..default()
        });

        commands.spawn_bundle(PbrBundle {
            mesh: plane.clone(),
            material: material.clone(),
            transform,
            ..default()
        })
        .id().id()
    };

    textures[1] = {
        material = materials.add(StandardMaterial {
            base_color_texture: Some(texture_handles[1].clone()),
            alpha_mode: AlphaMode::Blend,
            unlit: true,
            ..default()
        });

        commands.spawn_bundle(PbrBundle {
            mesh: plane.clone(),
            material: material.clone(),
            transform: Transform {
                translation: Vec3::new(x, y+0.5, z),
                ..default()
            },
            ..default()
        })
        .id().id()
    };

    textures[2] = {
        transform = Transform {
            translation: Vec3::new(x, y, z+0.5),
            rotation: Quat::from_rotation_x(std::f32::consts::PI / 2.0),
            ..default()
        };

        material = materials.add(StandardMaterial {
            base_color_texture: Some(texture_handles[2].clone()),
            alpha_mode: AlphaMode::Blend,
            unlit: true,
            ..default()
        });

        commands.spawn_bundle(PbrBundle {
            mesh: plane.clone(),
            material: material.clone(),
            transform,
            ..default()
        })
        .id().id()
    };

    textures[3] = {
        transform = Transform {
            translation: Vec3::new(x-0.5, y, z),
            rotation: Quat::from_rotation_x(-std::f32::consts::PI / 2.0),
            ..default()
        };
        transform.rotate(Quat::from_axis_angle(Vec3::Y, std::f32::consts::PI / 2.0));

        material = materials.add(StandardMaterial {
            base_color_texture: Some(texture_handles[3].clone()),
            alpha_mode: AlphaMode::Blend,
            unlit: true,
            ..default()
        });

        commands.spawn_bundle(PbrBundle {
            mesh: plane.clone(),
            material: material.clone(),
            transform,
            ..default()
        })
        .id().id()
    };

    textures[4] = {
        transform = Transform {
            translation: Vec3::new(x, y-0.5, z),
            rotation: Quat::from_rotation_x(std::f32::consts::PI),
            ..default()
        };

        material = materials.add(StandardMaterial {
            base_color_texture: Some(texture_handles[4].clone()),
            alpha_mode: AlphaMode::Blend,
            unlit: true,
            ..default()
        });

        commands.spawn_bundle(PbrBundle {
            mesh: plane.clone(),
            material: material.clone(),
            transform,
            ..default()
        })
        .id().id()
    };

    textures[5] = {
        transform = Transform {
            translation: Vec3::new(x, y, z-0.5),
            rotation: Quat::from_rotation_x(-std::f32::consts::PI / 2.0),
            ..default()
        };

        material = materials.add(StandardMaterial {
            base_color_texture: Some(texture_handles[5].clone()),
            alpha_mode: AlphaMode::Blend,
            unlit: true,
            ..default()
        });

        commands.spawn_bundle(PbrBundle {
            mesh: plane.clone(),
            material: material.clone(),
            transform,
            ..default()
        })
        .id().id()
    };

    commands.spawn()
        .insert_bundle(TransformBundle::from(Transform::from_translation(coord)))
        .insert(Block{ textures, coord })
        .insert(Collider::cuboid(0.5, 0.5, 0.5));
}