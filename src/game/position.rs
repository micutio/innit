use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Default, Clone, Copy)]
pub struct Position {
    x: i32,
    y: i32,
    last_x: i32,
    last_y: i32,
}

impl PartialEq for Position {
    fn eq(&self, other: &Self) -> bool {
        self.x == other.x && self.y == other.y
    }
}

impl From<Position> for rltk::Point {
    fn from(val: Position) -> Self {
        rltk::Point::new(val.x, val.y)
    }
}

impl From<rltk::Point> for Position {
    fn from(p: rltk::Point) -> Self {
        Position {
            x: p.x,
            y: p.y,
            last_x: p.x,
            last_y: p.y,
        }
    }
}

impl Position {
    pub fn from_xy(x: i32, y: i32) -> Self {
        // temporary sanity check
        if x > 80 || y > 60 {
            panic!("invalid postion ({}, {})", x, y);
        }
        Position {
            x,
            y,
            last_x: x,
            last_y: y,
        }
    }

    pub fn from_pos(pos: &Position) -> Self {
        Position {
            x: pos.x,
            y: pos.y,
            last_x: pos.x,
            last_y: pos.y,
        }
    }

    pub fn x(&self) -> i32 {
        self.x
    }

    pub fn y(&self) -> i32 {
        self.y
    }

    pub fn is_equal(&self, other: &Position) -> bool {
        self.x == other.x && self.y == other.y
    }

    pub fn _is_eq(&self, x: i32, y: i32) -> bool {
        self.x == x && self.y == y
    }

    pub fn move_to_xy(&mut self, a: i32, b: i32) {
        // temporary sanity check
        if a > 80 || b > 60 {
            panic!("invalid postion ({}, {})", a, b);
        }

        self.x = a;
        self.y = b;
    }

    pub fn move_to(&mut self, p: &Position) {
        // temporary sanity check
        if p.x > 80 || p.y > 60 {
            panic!("invalid postion ({}, {})", p.x, p.y);
        }

        self.x = p.x;
        self.y = p.y;
    }

    /// Return previous position if the it has changed since the last update, `None` otherwise.
    /// To be used by the `crate::game::ObjectStore`
    pub fn update(&mut self) -> Option<(i32, i32)> {
        let is_changed = self.x != self.last_x || self.y != self.last_y;
        let result = if is_changed {
            Some((self.last_x, self.last_y))
        } else {
            None
        };

        if is_changed {
            self.last_x = self.x;
            self.last_y = self.y;
        }

        result
    }

    pub fn is_adjacent(&self, other: &Position) -> bool {
        let delta_x = (other.x - self.x).abs();
        let delta_y = (other.y - self.y).abs();
        delta_x <= 1 && delta_y <= 1 && delta_x + delta_y == 1
    }

    pub fn offset(&self, other: &Position) -> (i32, i32) {
        (other.x - self.x, other.y - self.y)
    }

    pub fn translate(&mut self, delta: &Position) {
        self.move_to_xy(self.x + delta.x, self.y + delta.y);
    }

    pub fn get_translated(&self, delta: &Position) -> Position {
        Position::from_xy(self.x + delta.x, self.y + delta.y)
    }

    /// Return distance of this object to a given coordinate.
    pub fn distance(&self, other: &Position) -> f32 {
        let dx = (other.x - self.x) as f32;
        let dy = (other.y - self.y) as f32;
        f32::sqrt(dx * dx + dy * dy)
    }
}
