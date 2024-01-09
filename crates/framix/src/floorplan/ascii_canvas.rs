pub(crate) struct AsciiCanvas {
  width:  u32,
  height: u32,
  data:   Vec<char>,
}

impl AsciiCanvas {
  pub fn new(width: u32, height: u32) -> Self {
    Self {
      width,
      height,
      data: vec![' '; (width * height) as usize],
    }
  }

  pub fn set(&mut self, x: u32, y: u32, c: char) {
    self.data[(y * self.width + x) as usize] = c;
  }

  pub fn render(&self) -> String {
    self
      .data
      .chunks(self.width as usize)
      .map(|row| row.iter().collect::<String>())
      .collect::<Vec<_>>()
      .join("\n")
  }
}
