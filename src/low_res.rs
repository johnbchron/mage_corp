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
  pub pixel_size:  u8,
  pub window_res:  (u32, u32),
  pub texture_res: (u32, u32),
}

impl Default for LowResCamera {
  fn default() -> Self {
    Self {
      pixel_size:  8,
      window_res:  (1920, 1080),
      texture_res: (240, 135),
    }
  }
}

#[derive(Component, Default, Reflect)]
#[reflect(Component)]
pub struct LowResCameraTarget;

#[derive(Component, Default, Reflect)]
#[reflect(Component)]
pub struct LowResSecondaryCamera;

fn update_resolutions(
  mut camera_query: Query<&mut LowResCamera>,
  window_query: Query<&Window, With<PrimaryWindow>>,
) {
  let window = window_query.single();
  let mut camera = camera_query.single_mut();

  camera.window_res = (window.width() as u32, window.height() as u32);
  camera.texture_res = (
    (window.width() / camera.pixel_size as f32).ceil() as u32,
    (window.height() / camera.pixel_size as f32).ceil() as u32,
  );
}

fn maintain_texture_resolution(
  mut camera_query: Query<(&LowResCamera, &mut Camera)>,
  mut target_query: Query<Entity, With<LowResCameraTarget>>,
  mut commands: Commands,
  mut images: ResMut<Assets<Image>>,
  mut materials: ResMut<Assets<StandardMaterial>>,
) {
  let (lowres_camera, mut camera) = camera_query.single_mut();

  let image_size = Extent3d {
    width:                 lowres_camera.texture_res.0,
    height:                lowres_camera.texture_res.1,
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

  let image_handle = images.add(image);

  camera.target = RenderTarget::Image(image_handle.clone());
  camera.viewport = None;

  let texture_material = materials.add(StandardMaterial {
    base_color_texture: Some(image_handle),
    unlit: true,
    ..default()
  });

  let lowres_target = target_query.single_mut();

  commands.entity(lowres_target).insert(texture_material);
}

fn trigger_projection_remapping(
  window_query: Query<&Window, (With<PrimaryWindow>, Changed<Window>)>,
  mut projection_query: Query<(&mut Projection, &LowResCamera)>,
) {
  for _ in window_query.iter() {
    let (mut projection, lowres_camera) = projection_query.single_mut();
    projection.update(
      lowres_camera.texture_res.0 as f32,
      lowres_camera.texture_res.1 as f32,
    );
  }
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
      .add_systems(Update, maintain_texture_resolution)
      .add_systems(Update, update_resolutions)
      .add_systems(Update, trigger_projection_remapping);
  }
}
