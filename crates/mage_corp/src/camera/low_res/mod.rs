mod panorbit_compat;

use bevy::{
  core_pipeline::clear_color::ClearColorConfig,
  prelude::*,
  render::{
    camera::{CameraProjection, RenderTarget, ScalingMode},
    render_resource::{
      Extent3d, TextureDescriptor, TextureDimension, TextureFormat,
      TextureUsages,
    },
    view::RenderLayers,
  },
  window::PrimaryWindow,
};

#[derive(Component, Reflect)]
#[reflect(Component)]
pub struct LowResCamera {
  pub pixel_size: f32,
}

#[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
impl LowResCamera {
  /// Returns a u8 pixel size from the stored f32 pixel size.
  pub fn pixel_size(&self) -> u8 {
    (self.pixel_size.round() % f32::from(u8::MAX)) as u8
  }
}

impl Default for LowResCamera {
  fn default() -> Self {
    Self { pixel_size: 8.0 }
  }
}

#[derive(Component, Default, Reflect)]
#[reflect(Component)]
pub struct LowResCameraTarget;

// when the window gets resized, update the camera's resolution
pub fn calculate_texture_resolution(
  window_x: f32,
  window_y: f32,
  pixel_size: u8,
) -> UVec2 {
  UVec2::new(
    (window_x / f32::from(pixel_size)).ceil() as u32,
    (window_y / f32::from(pixel_size)).ceil() as u32,
  )
}

fn rebuild_texture_setup(
  mut camera_query: Query<(&LowResCamera, &mut Camera)>,
  target_query: Query<Entity, With<LowResCameraTarget>>,
  window_query: Query<&Window, With<PrimaryWindow>>,
  mut commands: Commands,
  mut images: ResMut<Assets<Image>>,
) {
  let (lowres_camera, mut camera) = camera_query.iter_mut().next().unwrap();
  let window = window_query.iter().next().unwrap();
  let lowres_target = target_query.iter().next().unwrap();

  let desired_texture_size = calculate_texture_resolution(
    window.width(),
    window.height(),
    lowres_camera.pixel_size(),
  );

  // if the camera already has a texture and it's the right size, use that
  if let RenderTarget::Image(image_handle) = &camera.target {
    let image = images.get(image_handle).unwrap();
    if image.size() == desired_texture_size {
      commands.entity(lowres_target).insert(image_handle.clone());
      return;
    }
  }

  // if we didn't find a texture, or the texture is the wrong size,
  // create a new texture
  let image =
    build_texture_image(desired_texture_size.x, desired_texture_size.y);
  let image_handle = images.add(image);
  // set the camera's target to the new texture
  camera.target = RenderTarget::Image(image_handle.clone());
  // add the new texture to the target
  commands.entity(lowres_target).insert(image_handle.clone());
}

fn trigger_projection_rescaling(
  window_query: Query<&Window, (With<PrimaryWindow>, Changed<Window>)>,
  mut projection_query: Query<(&mut Projection, &LowResCamera)>,
) {
  for window in window_query.iter() {
    let (mut projection, lowres_camera) = projection_query.single_mut();
    let desired_texture_size = calculate_texture_resolution(
      window.width(),
      window.height(),
      lowres_camera.pixel_size(),
    );
    projection
      .update(desired_texture_size.x as f32, desired_texture_size.y as f32);
  }
}

fn window_size_changed(
  window_q: Query<Entity, (With<PrimaryWindow>, Changed<Window>)>,
) -> bool {
  window_q.iter().next().is_some()
}

#[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
fn build_texture_image(x: u32, y: u32) -> Image {
  let image_size = Extent3d {
    width:                 x,
    height:                y,
    depth_or_array_layers: 1,
  };

  let mut image = Image {
    texture_descriptor: TextureDescriptor {
      label:           None,
      size:            image_size,
      dimension:       TextureDimension::D2,
      format:          TextureFormat::Bgra8UnormSrgb,
      mip_level_count: 1,
      sample_count:    1,
      usage:           TextureUsages::TEXTURE_BINDING
        | TextureUsages::COPY_DST
        | TextureUsages::RENDER_ATTACHMENT,
      view_formats:    &[],
    },
    ..default()
  };

  // fill image.data with zeroes
  image.resize(image_size);

  image
}

fn setup_target_camera(mut commands: Commands) {
  let second_pass_layer = RenderLayers::layer(1);

  commands.spawn((
    SpriteBundle {
      sprite: Sprite {
        custom_size: Some(Vec2::new(1.0, 1.0)),
        ..default()
      },
      ..default()
    },
    LowResCameraTarget,
    second_pass_layer,
    Name::new("lowres_target"),
  ));

  commands.spawn((
    Camera2dBundle {
      camera_2d: Camera2d {
        clear_color: ClearColorConfig::Custom(Color::rgb(0.0, 0.0, 0.0)),
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
  ));
}

pub struct LowResPlugin;

impl Plugin for LowResPlugin {
  fn build(&self, app: &mut App) {
    app
      .insert_resource(Msaa::Off)
      .add_systems(Startup, setup_target_camera)
      .add_systems(Update, rebuild_texture_setup.run_if(window_size_changed))
      .add_systems(
        Update,
        trigger_projection_rescaling.run_if(window_size_changed),
      )
      .add_plugins(panorbit_compat::LowResPanOrbitCompatPlugin)
      .register_type::<LowResCamera>()
      .register_type::<LowResCameraTarget>();
  }
}
