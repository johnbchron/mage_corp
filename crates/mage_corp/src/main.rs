mod camera;
mod materials;
mod test_scene;

use bevy::prelude::*;
use bevy_inspector_egui::quick::WorldInspectorPlugin;

fn main() {
  App::new()
    .add_plugins((
      DefaultPlugins.set(ImagePlugin::default_nearest()),
      camera::lowres::LowresCameraPlugin,
      materials::MaterialsPlugin,
      test_scene::TestScenePlugin,
      WorldInspectorPlugin::default(),
    ))
    .insert_resource(Msaa::Off)
    .run();
}
