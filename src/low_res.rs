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

// #[derive(Resource, Reflect)]
// pub struct DownscaledTexture(Handle<Image>);

#[derive(Component, Reflect)]
#[reflect(Component)]
pub struct LowResCamera {
  pub pixel_size: u8,
}

impl Default for LowResCamera {
  fn default() -> Self {
    Self { pixel_size: 8 }
  }
}

#[derive(Component, Default, Reflect)]
#[reflect(Component)]
pub struct LowResCameraTarget;

#[derive(Component, Default, Reflect)]
#[reflect(Component)]
pub struct LowResSecondaryCamera;

// when the window gets resized, update the camera's resolution
fn calculate_texture_resolution(
  window_x: f32,
  window_y: f32,
  pixel_size: u8,
) -> Vec2 {
  Vec2::new(
    (window_x / pixel_size as f32).ceil(),
    (window_y / pixel_size as f32).ceil(),
  )
}

fn rebuild_texture_setup(
  mut camera_query: Query<(&LowResCamera, &mut Camera)>,
  target_query: Query<Entity, With<LowResCameraTarget>>,
  window_query: Query<&Window, With<PrimaryWindow>>,
  mut commands: Commands,
  mut images: ResMut<Assets<Image>>,
  mut materials: ResMut<Assets<StandardMaterial>>,
) {
  let (lowres_camera, mut camera) = camera_query.single_mut();
  let window = window_query.single();

  let desired_texture_size = calculate_texture_resolution(
    window.width(),
    window.height(),
    lowres_camera.pixel_size,
  );

  // this will hold the handle to the image texture if it already exists
  let mut image_handle: Option<Handle<Image>> = None;

  // if the camera already has a texture and it's the right size, use that
  if let RenderTarget::Image(render_handle) = &camera.target {
    let image = images.get(render_handle).unwrap();
    if image.size().x == desired_texture_size.x as f32
      && image.size().y == desired_texture_size.y as f32
    {
      image_handle = Some(render_handle.clone());
    }
  }

  // if we didn't find a texture, or the texture is the wrong size, create a new
  // one
  if image_handle.is_none() {
    // create a new texture
    let image =
      build_texture_image(desired_texture_size.x, desired_texture_size.y);
    image_handle = Some(images.add(image));
    // set the camera's target to the new texture
    camera.target = RenderTarget::Image(image_handle.clone().unwrap().clone());
  }

  // create a material with the texture. this has to be done every frame
  // see https://github.com/bevyengine/bevy/issues/8341
  let texture_material = materials.add(StandardMaterial {
    base_color_texture: Some(image_handle.unwrap()),
    unlit: true,
    ..default()
  });

  // add the material to the quad
  let lowres_target = target_query.single();
  commands
    .entity(lowres_target)
    .insert(texture_material.clone());
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
      lowres_camera.pixel_size,
    );
    projection.update(desired_texture_size.x, desired_texture_size.y);
  }
}

fn build_texture_image(x: f32, y: f32) -> Image {
  let image_size = Extent3d {
    width:                 x as u32,
    height:                y as u32,
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

fn setup_target_camera(
  mut commands: Commands,
  mut meshes: ResMut<Assets<Mesh>>,
) {
  let second_pass_layer = RenderLayers::layer(1);

  // spawn quad to apply material to on second pass layer

  commands.spawn((
    MaterialMeshBundle::<StandardMaterial> {
      mesh: meshes
        .add(Mesh::try_from(shape::Quad::new(Vec2::new(1.0, 1.0))).unwrap()),
      // material: texture_material,
      transform: Transform::from_xyz(0.0, 0.0, -1.0),
      ..default()
    },
    second_pass_layer,
    LowResCameraTarget,
  ));

  // spawn camera for second layer to look at quad

  commands.spawn((
    Camera3dBundle {
      camera_3d: Camera3d {
        clear_color: ClearColorConfig::Custom(Color::rgb(0.0, 0.0, 0.0)),
        ..default()
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
      }
      .into(),
      camera: Camera {
        order: 1,
        ..default()
      },
      ..default()
    },
    second_pass_layer,
  ));
}

pub struct LowResPlugin;

impl Plugin for LowResPlugin {
  fn build(&self, app: &mut App) {
    app
      .add_systems(Startup, setup_target_camera)
      .add_systems(Update, rebuild_texture_setup)
      .add_systems(Update, trigger_projection_rescaling)
      .register_type::<LowResCamera>();
  }
}
