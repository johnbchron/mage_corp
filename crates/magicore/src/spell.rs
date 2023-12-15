mod types;

use bevy::prelude::*;
pub use types::*;

use super::{blueprint::ActiveBlueprint, source::Source};

pub struct SpellPlugin;

impl Plugin for SpellPlugin {
  fn build(&self, app: &mut App) {
    app
      .register_type::<SpellDescriptor>()
      .register_type::<ActiveSpell>()
      .register_type::<SourceLink>()
      .add_systems(
        Update,
        (
          init_spell,
          trigger_phase,
          state_phase,
          expend_phase,
          cleanup_spells,
        )
          .chain(),
      );
  }
}

fn init_spell(
  mut commands: Commands,
  spell_q: Query<(Entity, &SpellDescriptor, &SourceLink), Without<ActiveSpell>>,
) {
  for (entity, descriptor, source_link) in spell_q.iter() {
    commands
      .entity(entity)
      .insert(ActiveSpell::new(descriptor.clone(), source_link.0));
  }
}

fn trigger_phase(
  mut spell_q: Query<&mut ActiveSpell>,
  bluep_q: Query<&ActiveBlueprint>,
) {
  for mut spell in spell_q.iter_mut() {
    spell.update_triggers(&bluep_q);
  }
}

fn state_phase(
  mut commands: Commands,
  mut spell_q: Query<&mut ActiveSpell, With<SourceLink>>,
) {
  for mut spell in spell_q.iter_mut() {
    let commands = &mut commands;
    spell.update_states(commands);
  }
}

fn expend_phase(
  time: Res<Time>,
  mut source_q: Query<&mut Source>,
  spell_q: Query<&ActiveSpell>,
  mut bluep_q: Query<(&mut ActiveBlueprint, &SourceLink)>,
) {
  let time = time.into_inner();
  for spell in spell_q.iter() {
    spell.expend(&mut bluep_q, &mut source_q, time);
  }
}

fn cleanup_spells(
  mut commands: Commands,
  source_q: Query<&Source>,
  spell_q: Query<(Entity, &ActiveSpell)>,
) {
  for (entity, spell) in spell_q.iter() {
    if !spell.is_active() {
      info!("spell_cleanup: inactive");
      commands.entity(entity).despawn_recursive();
    } else if source_q.get(spell.source()).is_err() {
      warn!("spell_cleanup: source {:?} not found", spell.source());
      commands.entity(entity).despawn_recursive();
    }
  }
}
