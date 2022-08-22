use bevy::prelude::*;
use bevy_rapier3d::prelude::*;

use crate::player::SeenObject;

#[derive(Component, Clone, Copy)]
pub struct Block {
    pub textures: [Entity; 6],
    pub coord: Vec3
}

pub struct BlockBreaker {
    pub block: Block,
    pub block_id: Entity,
    breaking_textures: [Entity; 6],
    pub phase: u8,
    pub elapsed_time: f32
}

impl BlockBreaker {
    fn new(
        commands: &mut Commands,
        asset_server: Res<AssetServer>,
        mut meshes: ResMut<Assets<Mesh>>,
        mut materials: ResMut<Assets<StandardMaterial>>,
        block: Block,
        block_id: Entity
    ) -> Self {
        let plane = meshes.add(Mesh::from(shape::Plane{ size: 1.0 }));
        let texture_handle = asset_server.load("textures/block/breaking0.png");
        let material = materials.add(StandardMaterial {
            base_color_texture: Some(texture_handle.clone()),
            alpha_mode: AlphaMode::Blend,
            unlit: true,
            ..default()
        });
        let breaking_textures = BlockBreaker::create_box(commands, plane, material, block.coord);
        Self {
            block,
            block_id,
            breaking_textures,
            phase: 0,
            elapsed_time: 0.0
        }
    }
    
    fn initialize(&mut self, block: Block, entity: Entity) {
        self.block = block;
        self.block_id = entity;
        self.phase = 0;
        self.elapsed_time = 0.0;
    }
    
    fn update(
        &mut self,
        commands: &mut Commands,
        asset_server: Res<AssetServer>,
        mut meshes: ResMut<Assets<Mesh>>,
        mut materials: ResMut<Assets<StandardMaterial>>,
    ) {
        for texture in self.breaking_textures {
            commands.entity(texture).despawn();
        }
        let plane = meshes.add(Mesh::from(shape::Plane{ size: 1.0 }));
        let texture_handle = asset_server.load(&format!("textures/block/breaking{}.png", self.phase));
        let material = materials.add(StandardMaterial {
            base_color_texture: Some(texture_handle.clone()),
            alpha_mode: AlphaMode::Blend,
            unlit: true,
            ..default()
        });
        self.breaking_textures = BlockBreaker::create_box(commands, plane, material, self.block.coord);
    }
    
    fn create_box(
        commands: &mut Commands,
        mesh: Handle<Mesh>,
        material: Handle<StandardMaterial>,
        coord: Vec3
    ) -> [Entity; 6] {
        let mut transform;
        let mut breaking_textures = [Entity::from_raw(0); 6];
        let (x, y, z) = (coord.x, coord.y, coord.z);
        breaking_textures[0] = {
            transform = Transform {
                translation: Vec3::new(x+0.5, y, z),
                rotation: Quat::from_rotation_x(-std::f32::consts::PI / 2.0),
                ..default()
            };
            transform.rotate(Quat::from_axis_angle(Vec3::Y, -std::f32::consts::PI / 2.0));

            commands.spawn_bundle(PbrBundle {
                mesh: mesh.clone(),
                material: material.clone(),
                transform,
                ..default()
            }).id()
        };
        //==========
        breaking_textures[1] = commands.spawn_bundle(PbrBundle {
            mesh: mesh.clone(),
            material: material.clone(),
            transform: Transform::from_xyz(x, y+0.5, z),
            ..default()
        }).id();
        //==========
        breaking_textures[2] = {
            transform = Transform {
                translation: Vec3::new(x, y, z+0.5),
                rotation: Quat::from_rotation_x(std::f32::consts::PI / 2.0),
                ..default()
            };

            commands.spawn_bundle(PbrBundle {
                mesh: mesh.clone(),
                material: material.clone(),
                transform,
                ..default()
            }).id()
        };
        //==========
        breaking_textures[3] = {
            transform = Transform {
                translation: Vec3::new(x-0.5, y, z),
                rotation: Quat::from_rotation_x(-std::f32::consts::PI / 2.0),
                ..default()
            };
            transform.rotate(Quat::from_axis_angle(Vec3::Y, std::f32::consts::PI / 2.0));

            commands.spawn_bundle(PbrBundle {
                mesh: mesh.clone(),
                material: material.clone(),
                transform,
                ..default()
            }).id()
        };
        //==========
        breaking_textures[4] = {
            transform = Transform {
                translation: Vec3::new(x, y-0.5, z),
                rotation: Quat::from_rotation_x(std::f32::consts::PI),
                ..default()
            };

            commands.spawn_bundle(PbrBundle {
                mesh: mesh.clone(),
                material: material.clone(),
                transform,
                ..default()
            }).id()
        };
        //==========
        breaking_textures[5] = {
            transform = Transform {
                translation: Vec3::new(x, y, z-0.5),
                rotation: Quat::from_rotation_x(-std::f32::consts::PI / 2.0),
                ..default()
            };

            commands.spawn_bundle(PbrBundle {
                mesh: mesh.clone(),
                material: material.clone(),
                transform,
                ..default()
            }).id()
        };
        breaking_textures
    }
    
    fn break_block(&self, commands: &mut Commands) {
        for texture in self.block.textures {
            commands.entity(texture).despawn();
        }
        self.clean_up(commands);
        commands.entity(self.block_id).despawn();
    }
    
    fn clean_up(&self, commands: &mut Commands) {
        for texture in self.breaking_textures {
            commands.entity(texture).despawn();
        }
    }
}

pub fn control_block(
    mut commands: Commands,
    blockbreaker: Option<ResMut<BlockBreaker>>,
    time: Res<Time>,
    mouse: Res<Input<MouseButton>>,
    asset_server: Res<AssetServer>,
    meshes: ResMut<Assets<Mesh>>,
    materials: ResMut<Assets<StandardMaterial>>,
    seen_block: Query<(Entity, &Block), With<SeenObject>>
) {
    if let Ok((entity, block)) = seen_block.get_single() && mouse.pressed(MouseButton::Left) {
        if let Some(mut breaker) = blockbreaker {
            if breaker.block_id != entity {
                breaker.initialize(*block, entity)
            } else {
                breaker.elapsed_time += time.delta_seconds();
                if breaker.elapsed_time > 0.375 {
                    breaker.elapsed_time -= 0.375;
                    breaker.phase += 1;
                    if breaker.phase == 9 {
                        breaker.break_block(&mut commands);
                        commands.remove_resource::<BlockBreaker>();
                    } else {
                        breaker.update(&mut commands, asset_server, meshes, materials)
                    }
                }
            }
        } else {
            let breaker = BlockBreaker::new(&mut commands, asset_server, meshes, materials, *block, entity);
            commands.insert_resource(breaker);
        }
    } else if let Some(breaker) = blockbreaker {
        breaker.clean_up(&mut commands);
        commands.remove_resource::<BlockBreaker>()
    }
}

pub fn create_block(
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