mod camera;
mod materials;
mod test_scene;

use bevy::prelude::*;
use bevy_inspector_egui::quick::WorldInspectorPlugin;

fn main() {
  App::new()
    .add_plugins((
      DefaultPlugins
        .set(ImagePlugin::default_nearest())
        .set(WindowPlugin {
          primary_window: Some(Window {
            present_mode: bevy::window::PresentMode::Immediate,
            ..default()
          }),
          ..default()
        }),
      camera::lowres::LowresCameraPlugin,
      materials::MaterialsPlugin,
      test_scene::TestScenePlugin,
      WorldInspectorPlugin::default(),
    ))
    .insert_resource(Msaa::Off)
    .run();
}
