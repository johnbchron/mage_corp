mod material;

use bevy::{prelude::*, scene::SceneInstance};
pub use material::ToonMaterial;

pub struct ToonPlugin;

impl Plugin for ToonPlugin {
  fn build(&self, app: &mut App) {
    app
      .add_plugins(MaterialPlugin::<ToonMaterial>::default())
	    .register_asset_reflect::<ToonMaterial>()
      .add_systems(Update, convert_scene_materials)
      ;
  }
}

#[derive(Component)]
pub struct ConvertToToonMaterial;

pub fn convert_scene_materials(
  unloaded_instances: Query<
    (Entity, &SceneInstance),
    With<ConvertToToonMaterial>,
  >,
  handles: Query<(Entity, &Handle<StandardMaterial>)>,
  pbr_materials: Res<Assets<StandardMaterial>>,
  scene_manager: Res<SceneSpawner>,
  mut toon_materials: ResMut<Assets<ToonMaterial>>,
  mut cmds: Commands,
) {
  for (entity, instance) in unloaded_instances.iter() {
    if scene_manager.instance_is_ready(**instance) {
      cmds.entity(entity).remove::<ConvertToToonMaterial>();

      // Iterate over all entities in scene (once it's loaded)
      let handles =
        handles.iter_many(scene_manager.iter_instance_entities(**instance));
      for (entity, material_handle) in handles {
        let Some(material) = pbr_materials.get(material_handle) else {
          continue;
        };
        let toon_material = toon_materials.add(material.into());
        cmds
          .entity(entity)
          .insert(toon_material)
          .remove::<Handle<StandardMaterial>>();
      }
    }
  }
}
