use bevy::{
  diagnostic::{
    EntityCountDiagnosticsPlugin, FrameTimeDiagnosticsPlugin,
    LogDiagnosticsPlugin,
  },
  prelude::*,
};
use bevy_diagnostic_vertex_count::{
  VertexCountDiagnosticsPlugin, VertexCountDiagnosticsSettings,
};

pub struct DebugPlugin;

impl Plugin for DebugPlugin {
  fn build(&self, app: &mut App) {
    app
      .add_plugins(LogDiagnosticsPlugin::default())
      .add_plugins(FrameTimeDiagnosticsPlugin)
      .add_plugins(EntityCountDiagnosticsPlugin)
      .insert_resource(VertexCountDiagnosticsSettings { only_visible: true })
      .add_plugins(VertexCountDiagnosticsPlugin);
  }
}
