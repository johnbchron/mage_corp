use bevy::{ecs::query::QuerySingleError, prelude::*};
use interpolation::Ease;

use super::low_res::LowResCamera;
use crate::utils::f32_lerp;

#[derive(Debug)]
pub struct ControlledCameraParams {
  translation:        Vec3,
  looking_at:         (Vec3, Vec3),
  fov:                f32,
  low_res_pixel_size: f32,
}

// we explicitly ignore the `looking_at` field in comparisons
impl PartialEq for ControlledCameraParams {
  fn eq(&self, other: &Self) -> bool {
    self.translation == other.translation
      && self.fov == other.fov
      && self.low_res_pixel_size == other.low_res_pixel_size
  }
}

impl ControlledCameraParams {
  fn from_components(
    transform: &Transform,
    projection: &Projection,
    lowres_camera: &LowResCamera,
  ) -> Option<Self> {
    match projection {
      Projection::Perspective(projection) => Some(Self {
        translation:        transform.translation,
        // we don't have the information to construct this, but it's mostly
        // irrelevant bc we won't use it in comparison
        looking_at:         (Vec3::ZERO, Vec3::ZERO),
        fov:                projection.fov,
        low_res_pixel_size: lowres_camera.pixel_size,
      }),
      Projection::Orthographic(_) => None,
    }
  }

  fn apply(
    &self,
    transform: &mut Transform,
    projection: &mut Projection,
    lowres_camera: &mut LowResCamera,
  ) {
    if let Projection::Perspective(ref mut projection) = projection {
      projection.fov = self.fov;
    }
    *transform = Transform::from_translation(self.translation)
      .looking_at(self.looking_at.0, self.looking_at.1);
    lowres_camera.pixel_size = self.low_res_pixel_size;
  }

  fn lerp(from: &Self, to: &Self, s: f32) -> Self {
    Self {
      translation:        from.translation.lerp(to.translation, s),
      looking_at:         to.looking_at,
      fov:                f32_lerp(from.fov, to.fov, s),
      low_res_pixel_size: f32_lerp(
        from.low_res_pixel_size,
        to.low_res_pixel_size,
        s,
      ),
    }
  }
}

#[derive(Clone, PartialEq, Eq, Default, Reflect)]
pub enum CameraPureState {
  #[default]
  Disabled,
  OverShoulder,
  Isometric,
  TestState,
}

impl CameraPureState {
  fn correct_params(
    &self,
    target_transform: &Transform,
  ) -> Option<ControlledCameraParams> {
    match self {
      CameraPureState::Disabled => None,
      CameraPureState::OverShoulder => todo!(),
      CameraPureState::Isometric => Some(ControlledCameraParams {
        translation:        Vec3::new(0.0, 12.0, 16.0)
          + target_transform.translation,
        looking_at:         (target_transform.translation, Vec3::Y),
        fov:                0.3,
        low_res_pixel_size: 2.0,
      }),
      CameraPureState::TestState => Some(ControlledCameraParams {
        translation:        Vec3::new(-32.0, 32.0, 32.0)
          + target_transform.translation,
        looking_at:         (target_transform.translation, Vec3::Y),
        fov:                0.2,
        low_res_pixel_size: 4.0,
      }),
    }
  }
}

#[derive(Component, Clone, Reflect)]
#[reflect(Component)]
pub enum CameraState {
  InState(CameraPureState),
  Transition {
    from:     CameraPureState,
    to:       CameraPureState,
    progress: f32,
  },
}

impl Default for CameraState {
  fn default() -> Self {
    Self::InState(CameraPureState::default())
  }
}

impl CameraState {
  pub fn transition(&mut self, new_state: &CameraPureState) {
    match self.clone() {
      Self::InState(from) => {
        *self = Self::Transition {
          from,
          to: new_state.clone(),
          progress: 0.0,
        }
      }
      Self::Transition { from, progress, .. } => {
        *self = Self::Transition {
          from,
          to: new_state.clone(),
          progress,
        }
      }
    }
  }

  pub fn reverse(&mut self) -> Option<()> {
    match self.clone() {
      Self::InState(_) => None,
      Self::Transition { from, to, progress } => {
        *self = Self::Transition {
          from:     to,
          to:       from,
          progress: 1.0 - progress,
        };
        Some(())
      }
    }
  }
}

#[derive(Component, Default, Reflect)]
#[reflect(Component)]
pub struct CameraStateTarget;

#[derive(Resource, Reflect)]
#[reflect(Resource)]
pub struct CameraStateConfig {
  lerp_seconds: f32,
}

impl Default for CameraStateConfig {
  fn default() -> Self {
    Self { lerp_seconds: 1.0 }
  }
}

pub struct CameraStatePlugin;

impl Plugin for CameraStatePlugin {
  fn build(&self, app: &mut App) {
    app
      .add_systems(Update, maintain_state)
      .init_resource::<CameraStateConfig>()
      .register_type::<CameraStateConfig>()
      .register_type::<CameraState>()
      .register_type::<CameraStateTarget>();
  }
}

pub fn maintain_state(
  config: Res<CameraStateConfig>,
  mut camera_q: Query<
    (
      &mut CameraState,
      &mut Transform,
      &mut Projection,
      &mut LowResCamera,
    ),
    Without<CameraStateTarget>,
  >,
  target_q: Query<&Transform, With<CameraStateTarget>>,
  time: Res<Time>,
) {
  let target_transform = target_q.get_single();
  if target_transform.is_err() {
    match target_transform.unwrap_err() {
      QuerySingleError::NoEntities(_) => {
        warn!("no entities have a `CameraStateTarget`, aborting")
      }
      QuerySingleError::MultipleEntities(_) => {
        warn!("multiple entities have a `CameraStateTarget`, aborting")
      }
    };
    return;
  }
  let target_transform = target_transform.unwrap();

  // run through each camera
  for (
    camera_state,
    mut camera_transform,
    mut camera_projection,
    mut camera_lowres,
  ) in camera_q.iter_mut()
  {
    match camera_state.clone() {
      CameraState::Transition { from, to, progress } => {
        // if `from` and `to` are the same, just set the state to that.
        // also if either `from` or `to` are `Disabled`, it doesn't make sense
        // to run a transition so just end it.
        if from == to
          || from == CameraPureState::Disabled
          || to == CameraPureState::Disabled
        {
          *camera_state.into_inner() = CameraState::InState(to.clone());
          break;
        }

        // we're assuming that it's fine to unwrap these because the only reason
        // that they could be `None` is if they were `Disabled`.
        let from_params = from.correct_params(target_transform).unwrap();
        let to_params = to.correct_params(target_transform).unwrap();

        let actual_params = ControlledCameraParams::lerp(
          &from_params,
          &to_params,
          progress.cubic_in_out(), /* interpolation::cub_bez(
                                    *   &0.0_f32,
                                    *   &config.k_value,
                                    *   &(1.0 - config.k_value),
                                    *   &1.0_f32,
                                    *   &progress,
                                    * ), */
        );

        actual_params.apply(
          &mut camera_transform,
          &mut camera_projection,
          &mut camera_lowres,
        );
      }
      CameraState::InState(camera_state) => {
        let correct_params = camera_state.correct_params(target_transform);
        let actual_params = ControlledCameraParams::from_components(
          &camera_transform,
          &camera_projection,
          &camera_lowres,
        );

        // means the state == Disabled
        if correct_params.is_none() {
          break;
        }
        // means the camera state can't be coerced with just a &mut
        if actual_params.is_none() {
          error!(
            "failed to maintain camera state due to an invalid `Projection`"
          );
          break;
        }

        let correct_params = correct_params.unwrap();
        let actual_params = actual_params.unwrap();

        // apply the difference if needed
        if actual_params != correct_params {
          correct_params.apply(
            &mut camera_transform,
            &mut camera_projection,
            &mut camera_lowres,
          );
        }
      }
    }

    // finish the transition if it's ready
    if let CameraState::Transition { to, progress, .. } = camera_state.clone() {
      if progress >= 1.0 {
        *camera_state.into_inner() = CameraState::InState(to);
        break;
      }
    }

    // tick forward the transition progress
    if let CameraState::Transition {
      ref mut progress, ..
    } = camera_state.into_inner()
    {
      *progress = (*progress + time.delta_seconds() / config.lerp_seconds)
        .clamp(0.0, 1.0);
    }
  }
}
