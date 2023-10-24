use bevy::prelude::*;

#[derive(Component, Reflect)]
pub struct Source {
  max_chr_capacity: f32,
  max_chr_flow:     f32,
  stored_chr:       f32,
}

impl Default for Source {
  fn default() -> Self {
    Self {
      max_chr_capacity: 100.0,
      max_chr_flow:     10.0,
      stored_chr:       100.0,
    }
  }
}

impl Source {
  pub fn new(max_chr_capacity: f32, max_chr_flow: f32) -> Self {
    Self {
      max_chr_capacity,
      max_chr_flow,
      stored_chr: max_chr_capacity,
    }
  }

  /// Expend a given amount of choranum. The amount must be positive, and the
  /// total expended is returned. A source cannot expend more than it contains.
  pub fn expend(&mut self, amount: f32) -> f32 {
    // make sure amount is positive
    let amount = amount.max(0.0);
    let starting_amount = self.stored_chr;
    // only subtract down to zero
    self.stored_chr = (self.stored_chr - amount).max(0.0);
    // return the actual amount expended
    starting_amount - self.stored_chr
  }
}
