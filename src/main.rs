use bevy::{
    prelude::*,
    render::texture::ImageSettings,
};
use bevy_rapier3d::prelude::{
    *,
};

mod block;
mod debugger;
mod player;
mod sky;
mod utils;

use block::{
    Block,
    control_block
};
use debugger::{
    Debugger, update_debugger
};
use player::{
    setup_player,
    ground_event,
    player_update,
    player_eye
};
use sky::{
    AtmospherePlugin,
    AtmosphereTransform,
    material::Atmosphere,
};

/// This example shows various ways to configure texture materials in 3D
fn main() {
    App::new()
        .insert_resource(Msaa { samples: 4 })
        .insert_resource(Atmosphere::default())
        .insert_resource(AtmosphereTransform::default())
        .insert_resource(ImageSettings::default_nearest())
        .insert_resource(Debugger::default())
        .add_plugins(DefaultPlugins)
        .add_plugin(RapierPhysicsPlugin::<NoUserData>::default())
        //.add_plugin(RapierDebugRenderPlugin::default()) //collision debugging
        .add_plugin(AtmospherePlugin::default())
        .add_startup_system(setup_player)
        .add_startup_system(setup_environment)
        .add_startup_system(terrain_generation)
        .add_system(ground_event)
        .add_system(player_update)
        .add_system(update_debugger)
        .add_system(daylight_cycle)
        .add_system(player_eye.label("raycast"))
        .add_system(control_block.after("raycast"))
        .run();
}

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
    let mut textures: [Entity; 6] = [Entity::from_raw(0); 6];
    let mut transform: Transform;
    let mut material: Handle<StandardMaterial>;
    let binding = asset_server.load("textures/block/stone.png");
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
        .id()
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
            transform: Transform::from_xyz(x, y+0.5, z),
            ..default()
        })
        .id()
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
        .id()
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
        .id()
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
        .id()
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
        .id()
    };

    commands.spawn()
        .insert_bundle(TransformBundle::from(Transform::from_translation(coord)))
        .insert(Block{ textures, coord })
        .insert(Collider::cuboid(0.5, 0.5, 0.5));
}