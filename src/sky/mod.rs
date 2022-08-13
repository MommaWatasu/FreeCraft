pub mod material;

use bevy::{
    pbr::NotShadowCaster,
    prelude::*, asset::load_internal_asset,
};
use std::ops::Deref;
use material::*;

/// Sets up the atmosphere and the systems that control it
///
/// Follows the first camera it finds
#[derive(Default)]
pub struct AtmospherePlugin {
    pub resolution: u32,
}

/// Label for startup system that prepares skyboxes
pub const ATMOSPHERE_INIT: &'static str = "ATMOSPHERE_INIT";

impl Plugin for AtmospherePlugin {
    fn build(&self, app: &mut App) {
        load_internal_asset!(
            app,
            ATMOSPHERE_TYPES_SHADER_HANDLE,
            "shaders/types.wgsl",
            Shader::from_wgsl
        );

        load_internal_asset!(
            app,
            ATMOSPHERE_MATH_SHADER_HANDLE,
            "shaders/math.wgsl",
            Shader::from_wgsl
        );

        load_internal_asset!(
            app,
            ATMOSPHERE_MAIN_SHADER_HANDLE,
            "shaders/main.wgsl",
            Shader::from_wgsl
        );

        app.add_plugin(MaterialPlugin::<Atmosphere>::default());
        
        app.add_startup_system_to_stage(StartupStage::PostStartup, atmosphere_init.label(ATMOSPHERE_INIT));

        app.add_system(atmosphere_dynamic_sky);
    }
}

fn atmosphere_init(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut sky_materials: ResMut<Assets<Atmosphere>>,
    atmosphere: Option<Res<Atmosphere>>,
) {
    let atmosphere = match atmosphere {
        None => Atmosphere::default(),
        Some(c) => c.deref().clone(),
    };

    let atmosphere = sky_materials.add(atmosphere);

    commands
        .spawn_bundle(MaterialMeshBundle {
            mesh: meshes.add(Mesh::from(shape::Icosphere {
                radius: -100.0,
                subdivisions: 2,
            })),
            material: atmosphere,
            ..Default::default()
        })
        .insert(NotShadowCaster)
        .insert(Name::new("Sky Box"));
}

fn atmosphere_dynamic_sky(
    global_atmosphere: Res<Atmosphere>,
    atmosphere_query: Query<&Handle<Atmosphere>>,
    mut atmospheres: ResMut<Assets<Atmosphere>>,
) {
    if global_atmosphere.is_changed() {
        if let Some(atmosphere_handle) = atmosphere_query.iter().next() {
            if let Some(atmosphere) = atmospheres.get_mut(atmosphere_handle) {
                *atmosphere = global_atmosphere.deref().clone();
            }
        }
    }
}
