mod config;
mod regions;

use bevy::prelude::*;
use bevy_implicits::prelude::*;

use self::config::TerrainConfig;

#[derive(Component, Reflect, Default)]
#[reflect(Component)]
pub struct TerrainDetailTarget;

#[derive(Resource, Reflect)]
#[reflect(Resource)]
pub struct TerrainCurrentShape(#[reflect(ignore)] pub Shape);

impl Default for TerrainCurrentShape {
  fn default() -> Self {
    TerrainCurrentShape(Shape::new_expr(
      "(sqrt(square(x) + square(y + 5000) + square(z)) - 5000) + ((sin(x / \
       20.0) + sin(y / 20.0) + sin(z / 20.0)) * 4.0)",
    ))
  }
}

#[derive(Resource, Reflect, Default)]
#[reflect(Resource)]
pub struct TerrainGenerations {
  pub current: u32,
  pub next:    Vec<u32>,
}

impl TerrainGenerations {
  pub fn next(&self) -> u32 {
    u32::max(
      self.current + 1,
      self.next.iter().max().copied().unwrap_or(0) + 1,
    )
  }
}

#[derive(Component, Reflect)]
pub struct TerrainPiece {
  pub generation: u32,
}

#[derive(Event)]
pub struct TerrainTriggerRegeneration {
  pub target_location: Vec3,
}

pub struct TerrainPlugin;

impl Plugin for TerrainPlugin {
  fn build(&self, app: &mut App) {
    app
      .init_resource::<TerrainConfig>()
      .init_resource::<TerrainCurrentShape>()
      .init_resource::<TerrainGenerations>()
      .add_event::<TerrainTriggerRegeneration>()
      .add_systems(Update, (kickstart_terrain, create_generation).chain());
  }
}

fn kickstart_terrain(
  generations: Res<TerrainGenerations>,
  mut event_writer: EventWriter<TerrainTriggerRegeneration>,
  target_q: Query<&Transform, With<TerrainDetailTarget>>,
) {
  if generations.current == 0 && generations.next.is_empty() {
    if let Ok(transform) = target_q.get_single() {
      event_writer.send(TerrainTriggerRegeneration {
        target_location: transform.translation,
      });
    }
  }
}

fn create_generation(
  mut commands: Commands,
  mut generations: ResMut<TerrainGenerations>,
  mut event_reader: EventReader<TerrainTriggerRegeneration>,
  shape: Res<TerrainCurrentShape>,
  config: Res<TerrainConfig>,
  asset_server: Res<AssetServer>,
) {
  let Some(event) = event_reader.read().next() else {
    return;
  };

  let gen_id = generations.next();

  for (i, region) in regions::calculate_regions(&config, event.target_location)
    .into_iter()
    .enumerate()
  {
    let inputs = MesherInputs {
      shape: shape.0.clone(),
      region,
    };
    let path =
      bevy_implicits::asset_path(inputs).expect("failed to get mesh path");

    let handle: Handle<ImplicitMesh> = asset_server.load(path);
    commands.spawn((
      TerrainPiece { generation: gen_id },
      handle,
      Name::new(format!("terrain-{:03}-{:04}", gen_id, i)),
    ));
  }

  generations.next.push(gen_id);
}
