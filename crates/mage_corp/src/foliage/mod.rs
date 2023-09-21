use bevy::{
  prelude::*,
  render::primitives::Aabb,
  tasks::{AsyncComputeTaskPool, Task},
};
use bevy_xpbd_3d::prelude::Collider;
use planiscope::{
  cache::{CacheProvider, DiskCacheProvider},
  mesher::{FastSurfaceNetsMesher, MesherDetail, MesherInputs, MesherRegion},
  shape::Shape,
};

use crate::{
  materials::toon::ToonMaterial,
  utils::{
    bevy_mesh_from_pls_mesh,
    in_progress::{
      in_progress_asset_flusher, InProgressAsset, InProgressComponent,
    },
  },
};

#[derive(Component, Default, Reflect)]
#[reflect(Component)]
struct Foliage {
  /// The `planiscope::Shape` from which to generate the foliage mesh and
  /// collider.
  shape:    Shape,
  /// The material to grant the foliage entity after the mesh has been
  /// generated.
  material: Handle<ToonMaterial>,
  /// The bounding box over which to generate the mesh.
  aabb:     Aabb,
}

#[derive(Resource, Clone, Reflect)]
#[reflect(Resource)]
struct FoliageMeshConfig {
  /// How many meshing voxels to place within one world unit.
  voxels_per_unit: u32,
}

impl Default for FoliageMeshConfig {
  fn default() -> Self {
    Self { voxels_per_unit: 8 }
  }
}

pub struct FoliagePlugin;

impl Plugin for FoliagePlugin {
  fn build(&self, app: &mut App) {
    app
      .register_type::<Foliage>()
      .init_resource::<FoliageMeshConfig>()
      .add_systems(Startup, spawn_test_foliage)
      .add_systems(Update, start_foliage_tasks)
      .add_systems(Update, in_progress_asset_flusher::<Mesh>);
  }
}

fn spawn_test_foliage(
  mut commands: Commands,
  mut toon_materials: ResMut<Assets<ToonMaterial>>,
) {
  commands.spawn((
    SpatialBundle::from_transform(Transform::from_xyz(0.0, 0.0, 5.0)),
    Foliage {
      shape:    Shape::new_expr("sqrt(square(x) + square(y) + square(z)) - 1"),
      material: toon_materials.add(ToonMaterial {
        color: Color::rgb(1.0, 1.0, 0.0),
        ..default()
      }),
      aabb:     Aabb {
        center:       Vec3::ZERO.into(),
        half_extents: Vec3::new(2.0, 2.0, 2.0).into(),
      },
    },
    Name::new("foliage_test"),
  ));
}

fn start_foliage_tasks(
  mut commands: Commands,
  foliage_q: Query<
    (Entity, &Foliage),
    (Without<InProgressAsset<Mesh>>, Without<Handle<Mesh>>),
  >,
  foliage_mesh_config: Res<FoliageMeshConfig>,
) {
  let thread_pool = AsyncComputeTaskPool::get();

  for (entity, foliage) in foliage_q.iter() {
    let region = MesherRegion {
      position: foliage.aabb.center,
      scale:    foliage.aabb.half_extents,
      detail:   MesherDetail::Resolution(
        foliage_mesh_config.voxels_per_unit as f32,
      ),
      prune:    false,
    };
    let shape = foliage.shape.clone();

    let mesh_task = thread_pool.spawn(async move {
      bevy_mesh_from_pls_mesh(
        DiskCacheProvider::<FastSurfaceNetsMesher>::default()
          .get_mesh(&MesherInputs { shape, region })
          .unwrap(),
      )
    });

    commands
      .entity(entity)
      .insert(InProgressAsset(mesh_task))
      .insert(foliage.material.clone());
  }
}
