mod camera;
mod terrain;
mod test_scene;

use bevy::{pbr::wireframe::WireframePlugin, prelude::*};
use bevy_inspector_egui::quick::WorldInspectorPlugin;
use bevy_xpbd_3d::prelude as xpbd;
pub use common::{markers, materials};

fn main() {
  App::new()
    .add_plugins((
      bevy_implicits::ImplicitsAssetSourcePlugin,
      DefaultPlugins.set(ImagePlugin::default_nearest()),
      bevy_implicits::ImplicitsPlugin,
      xpbd::PhysicsPlugins::default(),
      xpbd::PhysicsDebugPlugin::default(),
      WorldInspectorPlugin::default(),
      bevy_panorbit_camera::PanOrbitCameraPlugin,
      WireframePlugin,
    ))
    .add_plugins((
      camera::lowres::LowresCameraPlugin,
      materials::MaterialsPlugin,
      terrain::TerrainPlugin,
      test_scene::TestScenePlugin,
      magicore::MagicPlugin,
      framix::RenderedModulePlugin,
    ))
    .insert_resource(Msaa::Off)
    .insert_resource(AmbientLight {
      brightness: 0.20,
      ..default()
    })
    .insert_resource(xpbd::PhysicsDebugConfig::joints(
      Some(Color::RED),
      Some(Color::BLUE),
    ))
    .run();
}
