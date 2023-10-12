pub mod shapes;

use bevy::{prelude::*, render::primitives::Aabb, tasks::AsyncComputeTaskPool};
use bevy_xpbd_3d::prelude::*;
use planiscope::{
  cache::{CacheProvider, DiskCacheProvider},
  mesher::{FastSurfaceNetsMesher, MesherDetail, MesherInputs, MesherRegion},
  shape::{builder::*, Shape},
};

use crate::{
  materials::toon::ToonMaterial,
  utils::{
    bevy_mesh_from_pls_mesh,
    in_progress::{
      in_progress_asset_flusher, in_progress_component_flusher,
      InProgressAsset, InProgressComponent,
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
  voxels_per_unit: u16,
}

impl Default for FoliageMeshConfig {
  fn default() -> Self {
    Self {
      voxels_per_unit: 24,
    }
  }
}

pub struct FoliagePlugin;

impl Plugin for FoliagePlugin {
  fn build(&self, app: &mut App) {
    app
      .register_type::<Foliage>()
      .init_resource::<FoliageMeshConfig>()
      .register_type::<FoliageMeshConfig>()
      .add_systems(Startup, spawn_test_foliage)
      .add_systems(Update, start_foliage_tasks)
      .add_systems(Update, in_progress_asset_flusher::<Mesh>)
      .add_systems(Update, in_progress_component_flusher::<Collider>);
  }
}

fn spawn_test_foliage(
  mut commands: Commands,
  mut toon_materials: ResMut<Assets<ToonMaterial>>,
) {
  let cylinder = cylinder(1.0, 2.0);
  let spline_points =
    vec![[0.0, 0.0, 0.0], [0.0, 2.0, 0.0], [1.0, 4.0, 0.0], [
      1.0, 6.0, 1.0,
    ]];
  let shape = catmull_rom_spline(cylinder, spline_points, 0.5);

  commands.spawn((
    SpatialBundle::from_transform(Transform::from_xyz(0.0, 0.0, 5.0)),
    Foliage {
      shape,
      material: toon_materials.add(ToonMaterial {
        color: Color::rgb(1.0, 1.0, 0.0),
        ..default()
      }),
      aabb: Aabb {
        center:       Vec3::ZERO.into(),
        half_extents: Vec3::new(2.0, 4.0, 2.0).into(),
      },
    },
    Name::new("foliage_test"),
    RigidBody::Static,
  ));
}

#[allow(clippy::type_complexity)]
fn start_foliage_tasks(
  mut commands: Commands,
  foliage_q: Query<
    (Entity, &Foliage),
    Or<(
      (
        Without<InProgressAsset<Mesh>>,
        Without<Handle<Mesh>>,
        Without<InProgressComponent<Collider>>,
        Without<Collider>,
      ),
      Or<(Added<Foliage>, Changed<Foliage>)>,
    )>,
  >,
  foliage_mesh_config: Res<FoliageMeshConfig>,
) {
  let thread_pool = AsyncComputeTaskPool::get();

  for (entity, foliage) in foliage_q.iter() {
    let inputs = MesherInputs {
      shape:  foliage.shape.clone(),
      region: MesherRegion {
        position: foliage.aabb.center,
        scale:    foliage.aabb.half_extents * 2.0,
        detail:   MesherDetail::Resolution(f32::from(
          foliage_mesh_config.voxels_per_unit,
        )),
        prune:    false,
      },
    };

    let mesh_task = thread_pool.spawn({
      let inputs = inputs.clone();
      async move {
        bevy_mesh_from_pls_mesh(
          DiskCacheProvider::<FastSurfaceNetsMesher>::default()
            .get_mesh(&inputs)
            .unwrap(),
        )
      }
    });

    // let collider_task = thread_pool.spawn({
    //   let inputs = inputs.clone();
    //   async move {
    //     Collider::from(
    //       DiskCacheProvider::<FastSurfaceNetsMesher>::default()
    //         .get_collider(&inputs)
    //         .unwrap(),
    //     )
    //   }
    // });

    commands
      .entity(entity)
      .insert(InProgressAsset(mesh_task))
      // .insert(InProgressComponent(collider_task))
      .insert(foliage.material.clone());
  }
}
