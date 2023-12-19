mod panorbit_compat;

use bevy::{
  core_pipeline::{
    clear_color::ClearColorConfig,
    prepass::{DepthPrepass, NormalPrepass},
  },
  prelude::*,
  render::{
    camera::{RenderTarget, ScalingMode},
    render_resource::Extent3d,
    view::RenderLayers,
  },
  window::{PrimaryWindow, WindowResized},
};

#[derive(Component, Debug, Reflect)]
pub struct LowresCamera {
  pub n_cameras:       u8,
  pub min_pixel_scale: u32,
  pub final_far:       Option<f32>,
}

impl LowresCamera {
  fn projection_for_index(
    &self,
    i: usize,
    overall_proj: &PerspectiveProjection,
  ) -> PerspectiveProjection {
    let total_max = 2_u32.pow(self.n_cameras as u32) - 1;
    let frac_near = (2_u32.pow(i as u32) - 1) as f32 / total_max as f32;
    let frac_far = (2_u32.pow((i + 1) as u32) - 1) as f32 / total_max as f32;
    let near =
      overall_proj.near + frac_near * (overall_proj.far - overall_proj.near);
    let mut far =
      overall_proj.near + frac_far * (overall_proj.far - overall_proj.near);
    if let Some(final_far) = self.final_far {
      if i == self.n_cameras as usize - 1 {
        far = far.max(final_far);
      }
    }

    PerspectiveProjection {
      near,
      far,
      ..overall_proj.clone()
    }
  }

  fn pixel_size_for_index(&self, i: usize) -> u32 {
    self.n_cameras as u32 - i as u32 - 1 + self.min_pixel_scale
  }
}

impl Default for LowresCamera {
  fn default() -> Self {
    Self {
      n_cameras:       4,
      min_pixel_scale: 2,
      final_far:       None,
    }
  }
}

#[derive(Bundle)]
pub struct LowresCameraBundle {
  pub lowres_camera:  LowresCamera,
  pub projection:     Projection,
  pub spatial_bundle: SpatialBundle,
  pub name:           Name,
}

impl Default for LowresCameraBundle {
  fn default() -> Self {
    Self {
      lowres_camera:  LowresCamera::default(),
      projection:     Projection::Perspective(PerspectiveProjection::default()),
      spatial_bundle: SpatialBundle::default(),
      name:           Name::new("lowres_camera"),
    }
  }
}

#[derive(Component)]
pub struct LowresSubCamera;

#[derive(Component)]
pub struct LowresTarget;

#[derive(Component)]
pub struct LowresTargetCamera;

pub struct LowresCameraPlugin;

impl Plugin for LowresCameraPlugin {
  fn build(&self, app: &mut App) {
    app
      .register_type::<LowresCamera>()
      .add_event::<RebuildEvent>()
      .add_systems(Update, (trigger_rebuild, rebuild_setup).chain())
      .add_plugins(panorbit_compat::LowResPanOrbitCompatPlugin);
  }
}

#[derive(Event, Default)]
pub struct RebuildEvent;

fn trigger_rebuild(
  mut event_writer: EventWriter<RebuildEvent>,
  changed_lowres_cameras: Query<(), Changed<LowresCamera>>,
  changed_projections: Query<(), (Changed<Projection>, With<LowresCamera>)>,
  mut resize_events: EventReader<WindowResized>,
) {
  if changed_lowres_cameras.iter().next().is_some() {
    debug!("triggering lowres camera rebuild due to LowresCamera change");
    event_writer.send(RebuildEvent);
  } else if changed_projections.iter().next().is_some() {
    debug!("triggering lowres camera rebuild due to Projection change");
    event_writer.send(RebuildEvent);
  } else if resize_events.read().next().is_some() {
    debug!("triggering lowres camera rebuild due to PrimaryWindow change");
    event_writer.send(RebuildEvent);
  }
}

#[allow(clippy::too_many_arguments)]
fn rebuild_setup(
  mut commands: Commands,
  mut event_reader: EventReader<RebuildEvent>,
  lowres_cameras: Query<(
    &LowresCamera,
    &Projection,
    Entity,
    Option<&Children>,
    Option<&NormalPrepass>,
    Option<&DepthPrepass>,
  )>,
  old_sub_cameras: Query<&LowresSubCamera>,
  old_targets: Query<Entity, With<LowresTarget>>,
  old_target_cameras: Query<Entity, With<LowresTargetCamera>>,
  primary_window: Query<&Window, With<PrimaryWindow>>,
  mut textures: ResMut<Assets<Image>>,
) {
  // info!("rebuilding lowres cameras");

  // exit if there are no lowres cameras
  let Ok((
    lowres_camera,
    lowres_camera_proj,
    lowres_camera_entity,
    children,
    normal_prepass,
    depth_prepass,
  )) = lowres_cameras.get_single()
  else {
    return;
  };
  let lowres_camera_proj = match lowres_camera_proj {
    Projection::Perspective(proj) => proj,
    _ => return,
  };

  // exit if there are no rebuild events
  if event_reader.read().next().is_none() {
    return;
  }

  // delete any existing sub cameras
  if let Some(children) = children {
    for child in children.iter() {
      if old_sub_cameras.get(*child).is_ok() {
        commands.entity(*child).despawn_recursive();
      }
    }
  }

  // delete the target quads if they exist
  for target in old_targets.iter() {
    commands.entity(target).despawn_recursive();
  }

  // delete the target camera if it exists
  for target_camera in old_target_cameras.iter() {
    commands.entity(target_camera).despawn_recursive();
  }

  // get the logical size of the window
  let window = primary_window.single();
  let window_size = Vec2::new(window.width(), window.height());

  // build the textures for the sub cameras
  let texture_handles = (0..lowres_camera.n_cameras)
    .map(|i| lowres_camera.pixel_size_for_index(i as usize))
    .map(|pixel_scale| (window_size / pixel_scale as f32).ceil())
    .map(|size| textures.add(build_texture(size.x as u32, size.y as u32)))
    .collect::<Vec<_>>();

  // spawn the sub cameras
  commands
    .entity(lowres_camera_entity)
    .with_children(|parent| {
      for (i, texture_handle) in texture_handles.iter().enumerate() {
        let texture_handle = texture_handle.clone();

        let mut sub_cam = parent.spawn((
          Camera3dBundle {
            camera: Camera {
              target: RenderTarget::Image(texture_handle.clone()),
              ..default()
            },
            projection: Projection::Perspective(
              lowres_camera.projection_for_index(i, lowres_camera_proj),
            ),
            camera_3d: Camera3d {
              clear_color: ClearColorConfig::Custom(Color::NONE),
              ..default()
            },
            ..default()
          },
          LowresSubCamera,
          Name::new(format!("lowres_sub_camera_{}", i)),
        ));

        // add prepasses if they exist
        if normal_prepass.is_some() {
          sub_cam.insert(NormalPrepass);
        }
        if depth_prepass.is_some() {
          sub_cam.insert(DepthPrepass);
        }
      }
    });

  // spawn target quads
  let second_pass_layer = RenderLayers::layer(1);
  for (i, handle) in texture_handles.iter().enumerate() {
    commands.spawn((
      SpriteBundle {
        sprite: Sprite {
          custom_size: Some(Vec2::new(1.0, 1.0)),
          ..default()
        },
        texture: handle.clone(),
        transform: Transform::from_xyz(0.0, 0.0, -(i as f32)),
        ..default()
      },
      LowresTarget,
      second_pass_layer,
      Name::new(format!("lowres_target_{}", i)),
    ));
  }

  // spawn the target camera
  commands.spawn((
    Camera2dBundle {
      camera_2d: Camera2d {
        clear_color: ClearColorConfig::Default,
      },
      transform: Transform::from_xyz(0.0, 0.0, 1.0)
        .looking_at(Vec3::default(), Vec3::Y),
      projection: OrthographicProjection {
        far: 10.0,
        scale: 1.0,
        scaling_mode: ScalingMode::Fixed {
          width:  1.0,
          height: 1.0,
        },
        ..default()
      },
      camera: Camera {
        order: 1,
        ..default()
      },
      ..default()
    },
    second_pass_layer,
    Name::new("lowres_output_camera"),
    LowresTargetCamera,
  ));
}

fn build_texture(x: u32, y: u32) -> Image {
  let image_size = Extent3d {
    width:                 x,
    height:                y,
    depth_or_array_layers: 1,
  };

  let mut image = Image {
    texture_descriptor: bevy::render::render_resource::TextureDescriptor {
      label:           Some("lowres_camera_texture"),
      size:            image_size,
      dimension:       bevy::render::render_resource::TextureDimension::D2,
      format:
        bevy::render::render_resource::TextureFormat::Bgra8UnormSrgb,
      mip_level_count: 1,
      sample_count:    1,
      usage:
        bevy::render::render_resource::TextureUsages::TEXTURE_BINDING
          | bevy::render::render_resource::TextureUsages::COPY_DST
          | bevy::render::render_resource::TextureUsages::RENDER_ATTACHMENT,
      view_formats:    &[],
    },
    ..default()
  };

  // fill image.data with zeroes
  image.resize(image_size);

  image
}

#[cfg(test)]
mod tests {
  use bevy::prelude::*;

  use super::*;

  #[test]
  fn near_and_far_fields_calculate_correctly() {
    let lowres_camera = LowresCamera {
      n_cameras:       3,
      min_pixel_scale: 2,
      final_far:       None,
    };
    let overall_proj = PerspectiveProjection {
      near: 0.0,
      far: 1.0,
      ..default()
    };
    let expected_configs = vec![
      (0.0..=(1.0 / 7.0), 4),
      ((1.0 / 7.0)..=(3.0 / 7.0), 3),
      ((3.0 / 7.0)..=(7.0 / 7.0), 2),
    ];

    for (i, expected_config) in expected_configs.iter().enumerate() {
      let config = lowres_camera.projection_for_index(i, &overall_proj);

      let (range, pixel_size) = expected_config;
      assert_eq!(config.near, *range.start());
      assert_eq!(config.far, *range.end());
      assert_eq!(lowres_camera.pixel_size_for_index(i), *pixel_size);
    }
  }
}
