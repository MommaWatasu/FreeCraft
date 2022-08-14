use bevy::prelude::*;

#[derive(Component)]
pub struct Block {
    pub textures: [u32; 6],
    pub coord: Vec3
}