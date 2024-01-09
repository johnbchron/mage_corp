mod ascii_canvas;
use ascii_canvas::AsciiCanvas;
use rand::Rng;

struct Room {
  x:        u32,
  y:        u32,
  width:    u32,
  height:   u32,
  is_final: bool,
}

impl Room {
  fn new(x: u32, y: u32, width: u32, height: u32) -> Self {
    Self {
      x,
      y,
      width,
      height,
      is_final: false,
    }
  }

  fn subdivide(self, on_x: bool, ratio: f32) -> (Room, Room) {
    let (width, height) = if on_x {
      let width = (self.width as f32 * ratio).floor() as u32;
      let height = self.height;
      (width, height)
    } else {
      let width = self.width;
      let height = (self.height as f32 * ratio).floor() as u32;
      (width, height)
    };

    let (room1, room2) = if on_x {
      let room1 = Room::new(self.x, self.y, width, height);
      let room2 = Room::new(self.x + width, self.y, self.width - width, height);
      (room1, room2)
    } else {
      let room1 = Room::new(self.x, self.y, width, height);
      let room2 =
        Room::new(self.x, self.y + height, width, self.height - height);
      (room1, room2)
    };

    (room1, room2)
  }
}

pub struct Root {
  width:  u32,
  height: u32,
  rooms:  Vec<Room>,
}

// all in meters
const ROOM_MIN_DIMENSION: u32 = 4;
const ROOM_MAX_DIMENSION: u32 = 10;
const ROOM_MIN_RATIO: f32 = 0.3;

impl Root {
  pub fn generate(width: u32, height: u32) -> Self {
    let mut rng = rand::thread_rng();
    let mut rooms = vec![Room::new(0, 0, width as u32, height as u32)];

    loop {
      let mut new_rooms = vec![];
      let mut subdiv_count = 0;

      for mut room in rooms {
        // don't subdivide if we're already final
        if room.is_final {
          new_rooms.push(room);
          continue;
        }

        // criteria for subdividing
        let can_subdivide_on_x = room.width >= ROOM_MIN_DIMENSION * 2
          && (room.width as f32 / room.height as f32) >= ROOM_MIN_RATIO;
        let can_subdivide_on_y = room.height >= ROOM_MIN_DIMENSION * 2
          && (room.height as f32 / room.width as f32) >= ROOM_MIN_RATIO;

        // don't subdivide if we can't subdivide on either axis
        if !can_subdivide_on_x && !can_subdivide_on_y {
          new_rooms.push(room);
          continue;
        }

        // randomly decide not to subdivide if we're under the max dimension
        if room.width <= ROOM_MAX_DIMENSION * 2
          && room.height <= ROOM_MAX_DIMENSION * 2
        {
          if rng.gen_bool(0.5) {
            room.is_final = true;
            new_rooms.push(room);
            continue;
          }
        }

        // subdivide on the axis with the greatest ratio
        let on_x = can_subdivide_on_x
          && (!can_subdivide_on_y
            || (room.width as f32 / room.height as f32)
              > (room.height as f32 / room.width as f32));

        let ratio = rng.gen_range(ROOM_MIN_RATIO..(1.0 - ROOM_MIN_RATIO));
        let (room1, room2) = room.subdivide(on_x, ratio);
        new_rooms.push(room1);
        new_rooms.push(room2);
        subdiv_count += 1;
      }

      rooms = new_rooms;
      if subdiv_count == 0 {
        break;
      }
    }

    Root {
      rooms,
      width: width as u32,
      height: height as u32,
    }
  }

  pub fn render(&self) -> String {
    let mut canvas = AsciiCanvas::new(self.width + 4, self.height + 4);
    for room in &self.rooms {
      // draw borders
      for x in room.x..(room.x + room.width) {
        canvas.set(x, room.y, '-');
        canvas.set(x, room.y + room.height - 1, '-');
      }
      for y in room.y..(room.y + room.height) {
        canvas.set(room.x, y, '|');
        canvas.set(room.x + room.width - 1, y, '|');
      }
    }
    canvas.render()
  }
}
