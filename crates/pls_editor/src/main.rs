use std::f32::consts::{FRAC_PI_4, PI};

use anyhow::{Error, Result};
use bevy::{
  prelude::*,
  tasks::{AsyncComputeTaskPool, Task},
};
use bevy_egui::{egui, EguiContexts, EguiPlugin};
use futures_lite::future;
use planiscope::{
  comp::{CompilationSettings, Composition},
  mesh::FullMesh,
  rhai::eval,
  shape::Shape,
};

fn main() {
  App::new()
    .add_plugins(DefaultPlugins)
    .add_plugins(EguiPlugin)
    // .add_plugins(bevy_panorbit_camera::PanOrbitCameraPlugin)
    .init_resource::<ModelMaterialHandle>()
    .init_resource::<UiSettings>()
    .init_resource::<UiCode>()
    .add_systems(Startup, configure_visuals_system)
    .add_systems(Startup, configure_ui_state_system)
    .add_systems(Startup, setup_3d_env)
    .add_systems(Update, ui_system)
    .add_systems(Update, spawn_compute_mesh_jobs)
    .add_systems(Update, handle_tasks)
    .add_systems(Update, animate_light_direction)
    .add_systems(Update, draw_gizmos)
    .run();
}

#[derive(Resource, Clone, PartialEq)]
struct UiSettings {
  name:          String,
  parsing_error: Option<String>,
  translate:     [f32; 3],
  scale:         [f32; 3],
  max_depth:     usize,
  min_depth:     usize,
  use_colors:    bool,
  smooth_normals: bool,
}

impl Default for UiSettings {
  fn default() -> Self {
    Self {
      name:          "shape_name".to_string(),
      parsing_error: None,
      translate:     [0.0, 0.0, 0.0],
      scale:         [5.0, 5.0, 5.0],
      max_depth:     6,
      min_depth:     0,
      use_colors:    true,
      smooth_normals: true,
    }
  }
}

#[derive(Default, Resource)]
struct UiCode(pub String);

#[derive(Component)]
struct ComputeMeshJob(Task<Result<Mesh>>);

#[derive(Component)]
struct CurrentModel;

#[derive(Resource, Deref)]
struct ModelMaterialHandle(Handle<StandardMaterial>);

impl FromWorld for ModelMaterialHandle {
  fn from_world(world: &mut World) -> Self {
    let mut materials = world
      .get_resource_mut::<Assets<StandardMaterial>>()
      .unwrap();
    ModelMaterialHandle(
      materials.add(StandardMaterial::from(Color::rgb(1.0, 1.0, 1.0))),
    )
  }
}

fn configure_visuals_system(mut contexts: EguiContexts) {
  contexts.ctx_mut().set_visuals(egui::Visuals {
    window_rounding: 0.0.into(),
    ..Default::default()
  });
}

fn configure_ui_state_system(
  mut ui_settings: ResMut<UiSettings>,
  mut ui_code: ResMut<UiCode>,
) {
  ui_settings.name = "shape_name".to_string();
  ui_code.0 = r#"[
  shape(
    sphere(1.0),
    [0.0, 0.0, 0.0]
  )
]"#.to_string();
}

fn ui_system(
  mut contexts: EguiContexts,
  mut ui_settings: ResMut<UiSettings>,
  mut ui_code: ResMut<UiCode>,
) {
  let ctx = contexts.ctx_mut();

  egui::SidePanel::left("side_panel")
    .default_width(400.0)
    .show(ctx, |ui| {
      ui.heading("Planiscope Editor");
      ui.separator();

      ui.horizontal(|ui| {
        ui.label("Shape Name: ");
        ui.text_edit_singleline(&mut ui_settings.name);
        ui.label(".pls");
      });

      ui.vertical(|ui| {
        ui.label("Shape Code: ");
        ui.code_editor(&mut ui_code.0);
      });

      ui.label(ui_settings.parsing_error.clone().unwrap_or("".to_string()));
      
      ui.separator();

      ui.label("Viewing Cube");
      ui.horizontal(|ui| {
        ui.label("Translate: ");
        ui.add(
          egui::DragValue::new(&mut ui_settings.translate[0])
            .speed(0.1)
            .clamp_range(-100.0..=100.0),
        );
        ui.add(
          egui::DragValue::new(&mut ui_settings.translate[1])
            .speed(0.1)
            .clamp_range(-100.0..=100.0),
        );
        ui.add(
          egui::DragValue::new(&mut ui_settings.translate[2])
            .speed(0.1)
            .clamp_range(-100.0..=100.0),
        );
      });
      ui.horizontal(|ui| {
        ui.label("Size: ");
        ui.add(
          egui::DragValue::new(&mut ui_settings.scale[0])
            .speed(0.1)
            .clamp_range(-100.0..=100.0),
        );
        ui.add(
          egui::DragValue::new(&mut ui_settings.scale[1])
            .speed(0.1)
            .clamp_range(-100.0..=100.0),
        );
        ui.add(
          egui::DragValue::new(&mut ui_settings.scale[2])
            .speed(0.1)
            .clamp_range(-100.0..=100.0),
        );
      });
      
      ui.separator();
      
      ui.label("Depth");
      ui.horizontal(|ui| {
        ui.label("Max: ");
        ui.add(
          egui::DragValue::new(&mut ui_settings.max_depth)
            .speed(0.1)
            .clamp_range(0..=10),
        );
      });
      ui.horizontal(|ui| {
        ui.label("Min: ");
        ui.add(
          egui::DragValue::new(&mut ui_settings.min_depth)
            .speed(0.1)
            .clamp_range(0..=10),
        );
      });
      
      ui.separator();
      
      ui.horizontal(|ui| {
        ui.checkbox(&mut ui_settings.use_colors, "Use Colors");
        ui.checkbox(&mut ui_settings.smooth_normals, "Smooth Normals");
      });
    });
    
    
}

fn setup_3d_env(mut commands: Commands, mut gizmo_config: ResMut<GizmoConfig>) {
  gizmo_config.depth_bias = -1.0;
  
  // lights
  commands.spawn(DirectionalLightBundle {
    directional_light: DirectionalLight {
      shadows_enabled: true,
      ..default()
    },
    ..default()
  });

  // camera
  commands.spawn((
    Camera3dBundle {
      transform: Transform::from_xyz(2.5, 5.0, 10.0)
        .looking_at(Vec3::new(0.0, 0.0, 0.0), Vec3::Y),
      ..default()
    },
    // bevy_panorbit_camera::PanOrbitCamera {
    //   focus: Vec3::ZERO,
    //   ..default()
    // },
  ));
}

fn animate_light_direction(
  time: Res<Time>,
  mut query: Query<&mut Transform, With<DirectionalLight>>,
) {
  for mut transform in &mut query {
    transform.rotation = Quat::from_euler(
      EulerRot::ZYX,
      0.0,
      time.elapsed_seconds() * PI / 5.0,
      -FRAC_PI_4,
    );
  }
}

fn draw_gizmos(mut gizmos: Gizmos) {
  // draw axes at origin
  gizmos.line(
    Vec3::ZERO,
    Vec3::X * 0.5,
    Color::rgb(1.0, 0.0, 0.0),
  );
  gizmos.line(
    Vec3::ZERO,
    Vec3::Y * 0.5,
    Color::rgb(0.0, 1.0, 0.0),
  );
  gizmos.line(
    Vec3::ZERO,
    Vec3::Z * 0.5,
    Color::rgb(0.0, 0.0, 1.0),
  );
}

fn compute_mesh(
  settings: UiSettings,
  shapes: Vec<(Shape, [f32; 3])>,
) -> Result<Mesh> {
  let mut composition = Composition::new();
  shapes.into_iter().for_each(|(shape, pos)| {
    composition.add_shape(shape, pos);
  });

  let smallest_scale_dim = settings
    .scale
    .iter()
    .min_by(|a, b| a.total_cmp(b))
    .ok_or(Error::msg("unable to find smallest scale axis"))?;
  let min_voxel_size =
    smallest_scale_dim * 2.0 / 2.0f32.powi(settings.max_depth as i32);

  let mut ctx = fidget::Context::new();
  let comp_settings = CompilationSettings { min_voxel_size };

  let solid_root_node = composition.compile_solid(&mut ctx, &comp_settings);
  let color_root_node = composition.compile_color(&mut ctx, &comp_settings);

  let solid_root_node = planiscope::nso::nso_normalize_region(
    solid_root_node,
    settings.translate,
    settings.scale,
    &mut ctx,
  );
  let color_root_node = planiscope::nso::nso_normalize_region(
    color_root_node,
    settings.translate,
    settings.scale,
    &mut ctx,
  );

  let solid_tape: fidget::eval::Tape<fidget::vm::Eval> =
    ctx.get_tape(solid_root_node).unwrap();
  let color_tape: fidget::eval::Tape<fidget::vm::Eval> =
    ctx.get_tape(color_root_node).unwrap();

  let mut full_mesh = FullMesh::tesselate(
    &solid_tape,
    if settings.use_colors {
      Some(&color_tape)
    } else {
      None
    },
    settings.smooth_normals,
    settings.max_depth.try_into()?,
    settings.min_depth.try_into()?,
  );

  full_mesh.prune();
  full_mesh.transform(settings.translate.into(), settings.scale.into());

  Ok(full_mesh.into())
}

fn spawn_compute_mesh_jobs(
  mut commands: Commands,
  mut settings: ResMut<UiSettings>,
  mut previous_settings: Local<UiSettings>,
  ui_code: Res<UiCode>,
  mut previous_code: Local<String>,
  previous_jobs: Query<Entity, With<ComputeMeshJob>>,
) {
  let pool = AsyncComputeTaskPool::get();

  if ui_code.0 != *previous_code || *previous_settings != *settings {
    let shape_code = ui_code.0.clone();

    for job in previous_jobs.iter() {
      commands.entity(job).despawn_recursive();
    }

    match eval(&shape_code) {
      Ok(shapes) => {
        settings.parsing_error = None;
        let ui_settings = settings.clone();
        let task = pool.spawn(async move { compute_mesh(ui_settings, shapes) });

        commands.spawn(ComputeMeshJob(task));
      }
      Err(error) => {
        settings.parsing_error = Some(error.to_string());
      }
    }
  }

  *previous_code = ui_code.0.clone();
  *previous_settings = settings.clone();
}

fn handle_tasks(
  mut commands: Commands,
  mut compute_mesh_jobs: Query<(Entity, &mut ComputeMeshJob)>,
  current_model: Query<Entity, With<CurrentModel>>,
  mut meshes: ResMut<Assets<Mesh>>,
  material: Res<ModelMaterialHandle>,
) {
  for (entity, mut task) in &mut compute_mesh_jobs {
    if let Some(Ok(mesh)) = future::block_on(future::poll_once(&mut task.0)) {
      // Despawn the previous model
      for old_model in current_model.iter() {
        commands.entity(old_model).despawn_recursive();
      }

      commands.entity(entity).despawn_recursive();

      commands.spawn((
        PbrBundle {
          mesh: meshes.add(mesh),
          material: material.clone(),
          ..default()
        },
        CurrentModel,
      ));
    }
  }
}
