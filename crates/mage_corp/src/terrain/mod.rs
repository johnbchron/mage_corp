use bevy::prelude::*;

pub struct TerrainRegion {
	pub position: Vec3,
	pub scale: Vec3,
}

pub struct TerrainMesh {
	mesh: Mesh,
	region: Vec<TerrainRegion>,
}