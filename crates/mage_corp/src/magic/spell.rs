use std::time::Duration;

use bevy::{prelude::*, utils::HashMap};
use nanorand::Rng;

use super::{blueprint::BlueprintDescriptor, source::Source};

#[derive(Clone, Default, Reflect)]
pub struct SpellBlock {
  init_trigger:     SpellTrigger,
  activate_trigger: SpellTrigger,
  end_trigger:      SpellTrigger,
  descriptors:      Vec<BlueprintDescriptor>,
}

impl SpellBlock {
  pub fn new(
    bluep_desc: Vec<BlueprintDescriptor>,
    init_trigger: SpellTrigger,
    activate_trigger: SpellTrigger,
    end_trigger: SpellTrigger,
  ) -> Self {
    Self {
      init_trigger,
      activate_trigger,
      end_trigger,
      descriptors: bluep_desc,
    }
  }
}

#[derive(Clone, Copy, Default, Reflect)]
pub enum BlockRef {
  Id(u64),
  #[default]
  SelfBlock,
}

#[derive(Clone, Default, Reflect)]
pub enum SpellTrigger {
  #[default]
  AtStart,
  OnBlockCompleted(BlockRef),
  AfterTime {
    #[reflect(ignore)]
    start:    Box<SpellTrigger>,
    duration: Duration,
  },
}

#[derive(Clone, Default, Reflect)]
pub enum BlockState {
  #[default]
  Uninit,
  Init(Vec<Entity>),
  Active(Vec<Entity>),
  End,
}

#[derive(Clone, Copy, Reflect)]
pub struct TriggerState {
  init:   bool,
  active: bool,
  built:  bool,
  end:    bool,
}

impl Default for TriggerState {
  fn default() -> Self {
    Self {
      init:   false,
      active: false,
      built:  false,
      end:    false,
    }
  }
}

#[derive(Component, Clone, Default, Reflect)]
pub struct SpellDescriptor {
  blocks: HashMap<u64, SpellBlock>,
}

impl SpellDescriptor {
  pub fn add(&mut self, block: SpellBlock) -> u64 {
    let mut rng = nanorand::tls_rng();
    let id = rng.generate::<u64>();
    self.blocks.insert(id, block);
    id
  }
}

#[derive(Clone, Default, Reflect)]
pub struct ActiveSpellBlock {
  block:         SpellBlock,
  trigger_state: TriggerState,
  block_state:   BlockState,
}

#[derive(Component, Clone, Reflect)]
pub struct ActiveSpell {
  descriptor:    SpellDescriptor,
  active_blocks: HashMap<u64, ActiveSpellBlock>,
  source:        Entity,
}

#[derive(Component, Clone, Reflect)]
pub struct SourceLink(pub Entity);

pub struct SpellPlugin;

impl Plugin for SpellPlugin {
  fn build(&self, app: &mut App) {
    app
      .register_type::<SpellDescriptor>()
      .register_type::<ActiveSpell>()
      .register_type::<SourceLink>()
      .add_systems(Update, init_spell)
      .add_systems(Update, check_for_disconnected_spells);
  }
}

fn init_spell(
  mut commands: Commands,
  spell_q: Query<(Entity, &SpellDescriptor, &SourceLink), Without<ActiveSpell>>,
) {
  for (entity, descriptor, source_link) in spell_q.iter() {
    let mut active_blocks = HashMap::default();
    for (id, block) in descriptor.blocks.iter() {
      active_blocks.insert(*id, ActiveSpellBlock {
        block:         block.clone(),
        trigger_state: TriggerState::default(),
        block_state:   BlockState::Uninit,
      });
    }
    let active_spell = ActiveSpell {
      descriptor: descriptor.clone(),
      active_blocks,
      source: source_link.0,
    };
    commands.entity(entity).insert(active_spell);
  }
}

fn check_for_disconnected_spells(
  mut commands: Commands,

  source_q: Query<&Source>,
  active_spell_q: Query<(Entity, &ActiveSpell), Changed<ActiveSpell>>,
) {
  for (spell_entity, active_spell) in active_spell_q.iter() {
    if source_q.get(active_spell.source).is_err() {
      error!(
        "Spell source entity was deleted, removing spell and spell entity"
      );
      commands.entity(spell_entity).despawn_recursive();
    }
  }
}

fn run_spell() {}
