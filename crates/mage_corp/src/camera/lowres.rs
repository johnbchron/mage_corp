use std::ops::RangeInclusive;

use bevy::{
  core_pipeline::clear_color::ClearColorConfig,
  prelude::*,
  render::{
    camera::{RenderTarget, ScalingMode},
    render_resource::Extent3d,
    view::RenderLayers,
  },
  window::PrimaryWindow,
};

#[derive(Component, Debug, Reflect)]
pub struct LowresCamera {
  /// A list of depth ranges and their corresponding resolution. The ranges
  /// should be in increasing order, and should cover the entire range [0.0,
  /// 1.0].
  pub configs:      Vec<(RangeInclusive<f32>, u8)>,
  pub overall_proj: PerspectiveProjection,
}

impl LowresCamera {
  /// Constructs a LowresCamera from a number of cameras, near and far.
  ///
  /// Each successive camera will have double the linear depth of the previous,
  /// and 1 pixel less resolution. The last camera will have 2-pixel resolution.
  pub fn from_n_cameras(n: u8, proj: PerspectiveProjection) -> Self {
    let total_max = 2_u32.pow(n as u32) - 1;
    let configs = (0..n)
      .map(|i| {
        let min = 2_u32.pow(i as u32) - 1;
        let max = 2_u32.pow((i + 1) as u32) - 1;
        let min = min as f32 / total_max as f32;
        let max = max as f32 / total_max as f32;
        (min..=max, n - i + 1)
      })
      .collect();
    Self {
      configs,
      overall_proj: proj,
    }
  }

  fn projection_for_index(&self, i: usize) -> PerspectiveProjection {
    let (range, _) = self.configs[i].clone();
    let mut proj = self.overall_proj.clone();

    proj.near = proj.near + (proj.far - proj.near) * range.start();
    proj.far = proj.near + (proj.far - proj.near) * range.end();

    proj
  }
}

impl Default for LowresCamera {
  fn default() -> Self {
    Self::from_n_cameras(4, PerspectiveProjection::default())
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
      .register_type::<RangeInclusive<f32>>()
      .add_systems(Update, build_setup);
  }
}

fn build_setup(
  mut commands: Commands,
  lowres_cameras: Query<
    (&LowresCamera, Entity, Option<&Children>),
    Changed<LowresCamera>,
  >,
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
  let texture_handles = lowres_camera
    .configs
    .iter()
    .map(|(_, res)| (window_size / *res as f32).ceil())
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
  fn from_n_cameras_works() {
    let lowres_camera =
      LowresCamera::from_n_cameras(3, PerspectiveProjection::default());
    let expected_configs = vec![
      (0.0..=(1.0 / 7.0), 4),
      ((1.0 / 7.0)..=(3.0 / 7.0), 3),
      ((3.0 / 7.0)..=(7.0 / 7.0), 2),
    ];
    assert_eq!(lowres_camera.configs, expected_configs);

    let lowres_camera =
      LowresCamera::from_n_cameras(4, PerspectiveProjection::default());
    let expected_configs = vec![
      (0.0..=(1.0 / 15.0), 5),
      ((1.0 / 15.0)..=(3.0 / 15.0), 4),
      ((3.0 / 15.0)..=(7.0 / 15.0), 3),
      ((7.0 / 15.0)..=(15.0 / 15.0), 2),
    ];
    assert_eq!(lowres_camera.configs, expected_configs);
  }
}
