use std::time::Duration;

use bevy::prelude::*;

use crate::utils::despawn::DespawnTag;

#[derive(Component, Reflect)]
#[reflect(Component)]
pub struct TimerLifetime {
  lifetime:  Duration,
  remaining: Duration,
}

impl Default for TimerLifetime {
  fn default() -> Self {
    Self::new(Duration::from_secs(1))
  }
}

impl TimerLifetime {
  pub fn new(lifetime: Duration) -> Self {
    Self {
      lifetime,
      remaining: lifetime,
    }
  }

  pub fn tick(&mut self, delta: Duration) {
    self.remaining = self.remaining.saturating_sub(delta);
  }

  pub fn is_expired(&self) -> bool {
    self.remaining <= Duration::ZERO
  }

  pub fn remaining_frac(&self) -> f32 {
    self.remaining.as_secs_f32() / self.lifetime.as_secs_f32()
  }
}

fn tick_timer_lifetimes(
  mut timer_lifetimes: Query<&mut TimerLifetime>,
  time: Res<Time>,
) {
  timer_lifetimes
    .par_iter_mut()
    .for_each(|mut timer_lifetime| {
      timer_lifetime.tick(time.delta());
    });
}

fn remove_expired_lifetimes(
  mut commands: Commands,
  timer_lifetimes: Query<(Entity, &TimerLifetime)>,
) {
  for (entity, timer_lifetime) in timer_lifetimes.iter() {
    if timer_lifetime.is_expired() {
      if let Some(mut entity_commands) = commands.get_entity(entity) {
        entity_commands.insert(DespawnTag);
      }
    }
  }
}

pub struct TimerLifetimePlugin;

impl Plugin for TimerLifetimePlugin {
  fn build(&self, app: &mut App) {
    app
      .add_systems(Update, tick_timer_lifetimes)
      .add_systems(
        Update,
        remove_expired_lifetimes.before(crate::utils::despawn::despawn),
      )
      .register_type::<TimerLifetime>();
  }
}
