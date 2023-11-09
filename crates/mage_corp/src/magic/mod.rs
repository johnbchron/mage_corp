pub mod blueprint;
pub mod source;
pub mod spell;
pub mod target;

use std::time::Duration;

use bevy::prelude::*;

use self::spell::{BlockRef, SpellBlock, SpellDescriptor, SpellTrigger};

pub struct MagicPlugin;

impl Plugin for MagicPlugin {
  fn build(&self, app: &mut App) {
    app
      .add_plugins((
        blueprint::BlueprintPlugin,
        source::SourcePlugin,
        spell::SpellPlugin,
        target::TargetPlugin,
      ))
      // .add_systems(Startup, magic_test_scene)
      .add_systems(Update, spawn_spell_test);
  }
}

// fn magic_test_scene(mut commands: Commands) {
//   commands.spawn((
//     SpatialBundle {
//       transform: Transform::from_xyz(0.0, 3.0, 3.0),
//       ..default()
//     },
//     ActiveBlueprint::new(&blueprint::BlueprintDescriptor::MassBarrier {
//       target: target::Target::RelativeCoords(Vec3::new(0.0, 0.0, 0.0)),
//       radius: 1.0,
//     }),
//     Name::new("blueprint_test"),
//   ));
// }

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
    vec![blueprint::BlueprintDescriptor::MassBarrier {
      target: target::Target::RelativeCoords(Vec3::new(0.0, 0.0, 0.0)),
      radius: 1.0,
    }],
    SpellTrigger::AtStart,
    SpellTrigger::AtStart,
    SpellTrigger::AfterTime {
      trigger:    Box::new(SpellTrigger::OnBlockActive(BlockRef::SelfBlock)),
      started_at: None,
      duration:   Duration::from_secs(10),
    },
  ));

  commands.spawn((
    spell_desc,
    spell::SourceLink(player),
    Name::new("spell_test"),
  ));
}
