use std::{sync::OnceLock, time::Duration};

use bevy::{
  prelude::*,
  utils::{HashMap, Instant},
};
use nanorand::Rng;
use thiserror::Error;

use super::super::blueprint::BlueprintDescriptor;
use crate::{blueprint::ActiveBlueprint, source::Source};

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

  fn triggers(&self) -> Vec<&SpellTrigger> {
    vec![
      &self.init_trigger,
      &self.activate_trigger,
      &self.end_trigger,
    ]
  }

  fn invalid_refs(&self, ids: &[u64]) -> Vec<BlockRef> {
    let mut invalid_refs = vec![];
    invalid_refs.extend(self.init_trigger.invalid_refs(ids));
    invalid_refs.extend(self.activate_trigger.invalid_refs(ids));
    invalid_refs.extend(self.end_trigger.invalid_refs(ids));
    invalid_refs
  }
}

#[derive(Clone, Copy, Debug, Default, Reflect)]
pub enum BlockRef {
  Id(u64),
  #[default]
  SelfBlock,
}

impl BlockRef {
  fn to_id(self, self_block_id: u64) -> u64 {
    match self {
      Self::Id(id) => id,
      Self::SelfBlock => self_block_id,
    }
  }

  fn is_valid(&self, ids: &[u64]) -> bool {
    match self {
      Self::Id(id) => ids.contains(id),
      Self::SelfBlock => true,
    }
  }
}

#[derive(Clone, Default, Reflect)]
pub enum SpellTrigger {
  #[default]
  AtStart,
  OnBlockInit(BlockRef),
  OnBlockBuilt(BlockRef),
  OnBlockActive(BlockRef),
  OnBlockEnd(BlockRef),
  AfterTime {
    #[reflect(ignore)]
    trigger:    Box<SpellTrigger>,
    started_at: Option<Instant>,
    duration:   Duration,
  },
}

impl SpellTrigger {
  fn update_if_needed(
    &mut self,
    _active_spell: &ActiveSpell,
    _self_block_id: u64,
  ) {
    if let Self::AfterTime {
      trigger,
      started_at,
      ..
    } = self
    {
      // mark the start time if we haven't already
      if started_at.is_none() {
        *started_at = Some(Instant::now());
      }
      // recurse because this trigger contains another trigger
      trigger.update_if_needed(_active_spell, _self_block_id);
    }
  }

  fn refs(&self) -> Vec<BlockRef> {
    match self {
      Self::AtStart => vec![],
      Self::OnBlockInit(block_ref)
      | Self::OnBlockBuilt(block_ref)
      | Self::OnBlockActive(block_ref)
      | Self::OnBlockEnd(block_ref) => vec![*block_ref],
      Self::AfterTime { trigger, .. } => trigger.refs(),
    }
  }

  fn invalid_refs(&self, ids: &[u64]) -> Vec<BlockRef> {
    self
      .refs()
      .into_iter()
      .filter(|r| !r.is_valid(ids))
      .collect::<Vec<_>>()
  }

  fn evaluate(
    &self,
    active_spell: &ActiveSpell,
    self_block_id: u64,
  ) -> Option<bool> {
    match self {
      // AtStart is always true
      Self::AtStart => Some(true),
      Self::OnBlockInit(block_ref) => {
        let id = block_ref.to_id(self_block_id);
        let block = active_spell.active_blocks.get(&id)?;
        Some(matches!(block.block_state, BlockState::Uninit))
      }
      Self::OnBlockBuilt(block_ref) => {
        let id = block_ref.to_id(self_block_id);
        let block = active_spell.active_blocks.get(&id)?;
        let built = block.trigger_state.built;
        match block.block_state {
          BlockState::Uninit => Some(false),
          BlockState::Init(_) => Some(built),
          _ => Some(false),
        }
      }
      Self::OnBlockActive(block_ref) => {
        let id = block_ref.to_id(self_block_id);
        let block = active_spell.active_blocks.get(&id)?;
        match block.block_state {
          BlockState::Uninit | BlockState::Init(_) => Some(false),
          _ => Some(true),
        }
      }
      Self::OnBlockEnd(block_ref) => {
        let id = block_ref.to_id(self_block_id);
        let block = active_spell.active_blocks.get(&id)?;
        Some(matches!(block.block_state, BlockState::End))
      }
      Self::AfterTime {
        trigger,
        started_at,
        duration,
      } => {
        if let Some(started_at) = started_at {
          // if the time has elapsed, evaluate the trigger
          if started_at.elapsed() >= *duration {
            Some(trigger.evaluate(active_spell, self_block_id)?)
          } else {
            Some(false)
          }
        } else {
          warn!(
            "tried to evaluate an AfterTime trigger before the start time was \
             marked"
          );
          None
        }
      }
    }
  }
}

#[derive(Clone, Default, Reflect)]
pub enum BlockState {
  #[default]
  Uninit,
  Init(Vec<Entity>),
  Active(Vec<Entity>),
  End,
}

#[derive(Clone, Copy, Reflect, Default)]
pub struct TriggerState {
  init:   bool,
  active: bool,
  built:  bool,
  end:    bool,
}

#[derive(Component, Clone, Default, Reflect)]
pub struct SpellDescriptor {
  blocks:   HashMap<u64, SpellBlock>,
  #[reflect(ignore)]
  is_valid: OnceLock<Result<Vec<SpellWarning>, SpellInvalidError>>,
}

impl SpellDescriptor {
  pub fn add(&mut self, block: SpellBlock) -> u64 {
    let mut rng = nanorand::tls_rng();
    let id = rng.generate::<u64>();

    self.add_with_id(id, block);

    id
  }

  pub fn add_with_id(&mut self, id: u64, block: SpellBlock) {
    self.blocks.insert(id, block);

    // invalidate the cached is_valid
    self.is_valid.take();
  }

  pub fn is_valid(&self) -> Result<Vec<SpellWarning>, SpellInvalidError> {
    self
      .is_valid
      .get_or_init(|| self.calculate_is_valid())
      .clone()
  }

  fn calculate_is_valid(&self) -> Result<Vec<SpellWarning>, SpellInvalidError> {
    if self.blocks.is_empty() {
      return Err(SpellInvalidError::NoBlocks);
    }

    let ids = self.sorted_block_ids();
    let invalid_refs = self
      .blocks
      .values()
      .flat_map(|b| b.invalid_refs(&ids))
      .collect::<Vec<_>>();

    if !invalid_refs.is_empty() {
      return Err(SpellInvalidError::InvalidBlockRef {
        containing_block: 0,
        block_ref:        invalid_refs[0],
      });
    }

    Ok(self.calculate_warnings())
  }

  fn calculate_warnings(&self) -> Vec<SpellWarning> {
    let mut warnings = vec![];

    for (id, block) in self.blocks.iter() {
      for trigger in block.triggers() {
        for block_ref in trigger.refs() {
          if let BlockRef::Id(block_id) = block_ref {
            if block_id == *id {
              warnings.push(SpellWarning::ExplicitSelfRef {
                containing_block: *id,
              });
            }
          }
        }
      }
    }

    warnings
  }

  fn sorted_block_ids(&self) -> Vec<u64> {
    let mut block_ids = self.blocks.keys().copied().collect::<Vec<_>>();
    block_ids.sort();
    block_ids
  }
}

#[derive(Error, Debug, Clone, Reflect)]
pub enum SpellInvalidError {
  #[error("Spell has no blocks")]
  NoBlocks,
  #[error(
    "Block {containing_block:#x} references an invalid block {block_ref:?}"
  )]
  InvalidBlockRef {
    containing_block: u64,
    block_ref:        BlockRef,
  },
}

#[derive(Error, Debug, Clone, Reflect)]
pub enum SpellWarning {
  #[error(
    "Block {containing_block:#x} has a trigger referencing itself with an \
     explicit ID"
  )]
  ExplicitSelfRef { containing_block: u64 },
}

#[derive(Clone, Default, Reflect)]
pub struct ActiveSpellBlock {
  descriptor:    SpellBlock,
  trigger_state: TriggerState,
  block_state:   BlockState,
}

#[derive(Component, Clone, Reflect)]
pub struct ActiveSpell {
  descriptor:    SpellDescriptor,
  active_blocks: HashMap<u64, ActiveSpellBlock>,
  source:        Entity,
}

impl ActiveSpell {
  pub fn new(descriptor: SpellDescriptor, source: Entity) -> Self {
    let mut active_blocks = HashMap::default();
    for (id, block) in descriptor.blocks.iter() {
      active_blocks.insert(*id, ActiveSpellBlock {
        descriptor:    block.clone(),
        trigger_state: TriggerState::default(),
        block_state:   BlockState::Uninit,
      });
    }
    Self {
      descriptor,
      active_blocks,
      source,
    }
  }

  pub fn descriptor(&self) -> &SpellDescriptor {
    &self.descriptor
  }
  pub fn source(&self) -> Entity {
    self.source
  }

  pub fn update_triggers(&mut self, bluep_q: &Query<&ActiveBlueprint>) {
    let block_ids = self.descriptor.sorted_block_ids();
    let old_state = self.clone();

    block_ids.into_iter().for_each(|id| {
      let block = self.active_blocks.get_mut(&id).unwrap();

      // calculate whether each blueprint is fully saturated
      let all_bluep_saturated = match &block.block_state {
        BlockState::Init(bluep) => bluep
          .iter()
          .all(|e| bluep_q.get(*e).is_ok_and(|b| b.saturated())),
        _ => false,
      };

      // update the triggers (for things like `started_at` timers)
      block
        .descriptor
        .init_trigger
        .update_if_needed(&old_state, id);
      block
        .descriptor
        .activate_trigger
        .update_if_needed(&old_state, id);
      block
        .descriptor
        .end_trigger
        .update_if_needed(&old_state, id);

      // calculate the trigger state for this block
      block.trigger_state = TriggerState {
        init:   block
          .descriptor
          .init_trigger
          .evaluate(&old_state, id)
          .unwrap_or(false),
        active: block
          .descriptor
          .activate_trigger
          .evaluate(&old_state, id)
          .unwrap_or(false),
        built:  all_bluep_saturated,
        end:    block
          .descriptor
          .end_trigger
          .evaluate(&old_state, id)
          .unwrap_or(false),
      };
    });
  }

  pub fn update_states(&mut self, commands: &mut Commands) {
    let block_ids = self.descriptor.sorted_block_ids();

    block_ids.into_iter().for_each(|id| {
      let block = self.active_blocks.get_mut(&id).unwrap();

      // handle the block state transitions
      match block.block_state.clone() {
        BlockState::Uninit => {
          if block.trigger_state.init {
            // spawn blueprint entities
            let entities = block
              .descriptor
              .descriptors
              .iter()
              .enumerate()
              .map(|(idx, d)| {
                ActiveBlueprint::new(d).spawn(
                  commands,
                  &SourceLink(self.source),
                  &format!("blueprint_{:#x}_{}", id, idx),
                )
              })
              .collect::<Vec<_>>();

            block.block_state = BlockState::Init(entities);
          }
        }
        BlockState::Init(tracked_bluep) => {
          if block.trigger_state.active && block.trigger_state.built {
            block.block_state = BlockState::Active(tracked_bluep);
          }
        }
        BlockState::Active(tracked_bluep) => {
          if block.trigger_state.end {
            tracked_bluep.into_iter().for_each(|e| {
              commands.entity(e).despawn_recursive();
            });

            block.block_state = BlockState::End;
          }
        }
        BlockState::End => {}
      }
    });
  }

  pub fn expend(
    &self,
    bluep_q: &mut Query<(&mut ActiveBlueprint, &SourceLink)>,
    source_q: &mut Query<&mut Source>,
    time: &Time,
  ) {
    let block_ids = self.descriptor.sorted_block_ids();

    block_ids.into_iter().for_each(|id| {
      let block = self.active_blocks.get(&id).unwrap();

      if let BlockState::Init(bluep) = &block.block_state {
        bluep.iter().for_each(|e| {
          // this entity will not exist the first frame after it's spawned
          if let Ok((mut bluep, source_link)) = bluep_q.get_mut(*e) {
            let mut source = source_q.get_mut(source_link.0).unwrap();

            let amount =
              (source.max_flow() * time.delta_seconds()).min(bluep.remaining());

            source.expend_to(amount, &mut bluep);
          }
        });
      }
    });
  }

  pub fn is_active(&self) -> bool {
    self.active_blocks.values().any(|b| {
      matches!(b.block_state, BlockState::Init(_) | BlockState::Active(_))
    })
  }
}

#[derive(Component, Clone, Reflect)]
pub struct SourceLink(pub Entity);
