use bevy::{
  core_pipeline::clear_color::ClearColorConfig,
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
  pub overall_proj:    PerspectiveProjection,
}

impl LowresCamera {
  fn projection_for_index(&self, i: usize) -> PerspectiveProjection {
    let total_max = 2_u32.pow(self.n_cameras as u32) - 1;
    let frac_near = (2_u32.pow(i as u32) - 1) as f32 / total_max as f32;
    let frac_far = (2_u32.pow((i + 1) as u32) - 1) as f32 / total_max as f32;
    let near = self.overall_proj.near
      + frac_near * (self.overall_proj.far - self.overall_proj.near);
    let far = self.overall_proj.near
      + frac_far * (self.overall_proj.far - self.overall_proj.near);

    PerspectiveProjection {
      near,
      far,
      ..self.overall_proj.clone()
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
      overall_proj:    PerspectiveProjection::default(),
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
      .add_systems(Update, (trigger_rebuild, rebuild_setup).chain());
  }
}

#[derive(Event, Default)]
struct RebuildEvent;

fn trigger_rebuild(
  mut event_writer: EventWriter<RebuildEvent>,
  lowres_cameras: Query<(), Changed<LowresCamera>>,
  mut resize_events: EventReader<WindowResized>,
) {
  if lowres_cameras.iter().next().is_some() {
    debug!("triggering lowres camera rebuild due to LowresCamera change");
    event_writer.send(RebuildEvent::default());
  } else if resize_events.read().next().is_some() {
    debug!("triggering lowres camera rebuild due to PrimaryWindow change");
    event_writer.send(RebuildEvent::default());
  }
}

fn rebuild_setup(
  mut commands: Commands,
  mut event_reader: EventReader<RebuildEvent>,
  lowres_cameras: Query<(&LowresCamera, Entity, Option<&Children>)>,
  old_sub_cameras: Query<&LowresSubCamera>,
  old_targets: Query<Entity, With<LowresTarget>>,
  old_target_cameras: Query<Entity, With<LowresTargetCamera>>,
  primary_window: Query<&Window, With<PrimaryWindow>>,
  mut textures: ResMut<Assets<Image>>,
) {
  // exit if there are no lowres cameras
  let Ok((lowres_camera, lowres_camera_entity, children)) =
    lowres_cameras.get_single()
  else {
    return;
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
      for i in 0..texture_handles.len() {
        let texture_handle = texture_handles[i].clone();

        parent.spawn((
          Camera3dBundle {
            camera: Camera {
              target: RenderTarget::Image(texture_handle),
              ..default()
            },
            projection: Projection::Perspective(
              lowres_camera.projection_for_index(i),
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
      label:           None,
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
      overall_proj:    PerspectiveProjection {
        near: 0.0,
        far: 1.0,
        ..default()
      },
    };
    let expected_configs = vec![
      (0.0..=(1.0 / 7.0), 4),
      ((1.0 / 7.0)..=(3.0 / 7.0), 3),
      ((3.0 / 7.0)..=(7.0 / 7.0), 2),
    ];

    for (i, expected_config) in expected_configs.iter().enumerate() {
      let config = lowres_camera.projection_for_index(i);

      let (range, pixel_size) = expected_config;
      assert_eq!(config.near, *range.start());
      assert_eq!(config.far, *range.end());
      assert_eq!(lowres_camera.pixel_size_for_index(i), *pixel_size);
    }
  }
}
