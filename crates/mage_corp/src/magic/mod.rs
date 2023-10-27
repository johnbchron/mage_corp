pub mod blueprint;
pub mod source;
pub mod spell;

use bevy::prelude::*;
use blueprint::ActiveBlueprint;

use self::spell::{BlockRef, SpellBlock, SpellDescriptor, SpellTrigger};

pub struct MagicPlugin;

impl Plugin for MagicPlugin {
  fn build(&self, app: &mut App) {
    app
      .add_plugins((blueprint::BlueprintPlugin, spell::SpellPlugin))
      .add_systems(Startup, magic_test_scene)
      .add_systems(Update, spawn_spell_test);
  }
}

fn magic_test_scene(mut commands: Commands) {
  commands.spawn((
    SpatialBundle {
      transform: Transform::from_xyz(0.0, 3.0, 3.0),
      ..default()
    },
    ActiveBlueprint::new(blueprint::BlueprintDescriptor::MassBarrier),
    Name::new("blueprint_test"),
  ));
}

fn spawn_spell_test(
  mut commands: Commands,
  player_q: Query<Entity, With<crate::markers::Player>>,
  keys: Res<Input<KeyCode>>,
) {
  if !keys.just_pressed(KeyCode::T) {
    return;
  }

  let player = player_q.single();

  let mut spell_desc = SpellDescriptor::default();
  spell_desc.add(SpellBlock::new(
    vec![blueprint::BlueprintDescriptor::MassBarrier],
    SpellTrigger::AtStart,
    SpellTrigger::AtStart,
    SpellTrigger::OnBlockCompleted(BlockRef::SelfBlock),
  ));

  commands.spawn((
    spell_desc,
    spell::SourceLink(player),
    Name::new("spell_test"),
  ));
}
