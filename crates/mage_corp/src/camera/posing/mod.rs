mod params;

use bevy::{ecs::query::QuerySingleError, prelude::*};
use interpolation::Ease;

use self::params::ControlledCameraParams;
use super::low_res::LowResCamera;

#[derive(Clone, PartialEq, Eq, Default, Reflect)]
pub enum CameraPose {
  #[default]
  Disabled,
  OverShoulder,
  Isometric,
  TestState,
}

impl CameraPose {
  fn correct_params(
    &self,
    target_transform: &Transform,
  ) -> Option<ControlledCameraParams> {
    match self {
      CameraPose::Disabled => None,
      CameraPose::OverShoulder => todo!(),
      CameraPose::Isometric => Some(ControlledCameraParams {
        translation:        Vec3::new(0.0, 12.0, 16.0)
          + target_transform.translation,
        looking_at:         (target_transform.translation, Vec3::Y),
        fov:                0.3,
        low_res_pixel_size: 2.0,
      }),
      CameraPose::TestState => Some(ControlledCameraParams {
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
pub enum CameraPoseState {
  InState(CameraPose),
  Transition {
    from:     CameraPose,
    to:       CameraPose,
    progress: f32,
  },
}

impl Default for CameraPoseState {
  fn default() -> Self {
    Self::InState(CameraPose::default())
  }
}

impl CameraPoseState {
  pub fn transition(&mut self, new_state: &CameraPose) {
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
pub struct CameraPoseConfig {
  lerp_seconds: f32,
}

impl Default for CameraPoseConfig {
  fn default() -> Self {
    Self { lerp_seconds: 1.0 }
  }
}

pub struct CameraPosePlugin;

impl Plugin for CameraPosePlugin {
  fn build(&self, app: &mut App) {
    app
      .add_systems(Update, maintain_pose)
      .init_resource::<CameraPoseConfig>()
      .register_type::<CameraPoseConfig>()
      .register_type::<CameraPoseState>()
      .register_type::<CameraStateTarget>();
  }
}

pub fn maintain_pose(
  config: Res<CameraPoseConfig>,
  mut camera_q: Query<
    (
      &mut CameraPoseState,
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
  if let Err(single_error) = target_transform {
    match single_error {
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
      CameraPoseState::Transition { from, to, progress } => {
        // if `from` and `to` are the same, just set the state to that.
        // also if either `from` or `to` are `Disabled`, it doesn't make sense
        // to run a transition so just end it.
        if from == to
          || from == CameraPose::Disabled
          || to == CameraPose::Disabled
        {
          *camera_state.into_inner() = CameraPoseState::InState(to.clone());
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
      CameraPoseState::InState(camera_state) => {
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
    if let CameraPoseState::Transition { to, progress, .. } =
      camera_state.clone()
    {
      if progress >= 1.0 {
        *camera_state.into_inner() = CameraPoseState::InState(to);
        break;
      }
    }

    // tick forward the transition progress
    if let CameraPoseState::Transition {
      ref mut progress, ..
    } = camera_state.into_inner()
    {
      *progress = (*progress + time.delta_seconds() / config.lerp_seconds)
        .clamp(0.0, 1.0);
    }
  }
}
