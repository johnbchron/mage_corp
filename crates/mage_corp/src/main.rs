mod camera;
mod materials;
mod terrain;
mod test_scene;

use bevy::prelude::*;
use bevy_inspector_egui::quick::WorldInspectorPlugin;
use bevy_xpbd_3d::prelude as xpbd;

fn main() {
  App::new()
    .add_plugins((
      bevy_implicits::ImplicitsAssetSourcePlugin,
      DefaultPlugins.set(ImagePlugin::default_nearest()),
      bevy_implicits::ImplicitsPlugin,
      xpbd::PhysicsPlugins::default(),
      WorldInspectorPlugin::default(),
    ))
    .add_plugins((
      camera::lowres::LowresCameraPlugin,
      materials::MaterialsPlugin,
      terrain::TerrainPlugin,
      test_scene::TestScenePlugin,
    ))
    .insert_resource(Msaa::Off)
    .run();
}
