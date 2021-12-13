use rltk;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Default, PartialEq, Clone, Copy)]
pub struct Position {
    pub x: i32,
    pub y: i32,
}

impl Into<rltk::Point> for Position {
    fn into(self) -> rltk::Point {
        rltk::Point::new(self.x, self.y)
    }
}

impl From<rltk::Point> for Position {
    fn from(p: rltk::Point) -> Self {
        Position { x: p.x, y: p.y }
    }
}

impl Position {
    pub fn new(x: i32, y: i32) -> Self {
        // temporary sanity check
        if x > 80 || y > 60 {
            panic!("invalid postion ({}, {})", x, y);
        }
        Position { x, y }
    }

    pub fn is_equal(&self, other: &Position) -> bool {
        self.x == other.x && self.y == other.y
    }

    pub fn is_eq(&self, x: i32, y: i32) -> bool {
        self.x == x && self.y == y
    }

    pub fn set(&mut self, a: i32, b: i32) {
        // temporary sanity check
        if a > 80 || b > 60 {
            panic!("invalid postion ({}, {})", a, b);
        }

        self.x = a;
        self.y = b;
    }

    pub fn is_adjacent(&self, other: &Position) -> bool {
        let delta_x = (other.x - self.x).abs();
        let delta_y = (other.y - self.y).abs();
        delta_x <= 1 && delta_y <= 1 && delta_x + delta_y == 1
    }

    pub fn offset(&self, other: &Position) -> (i32, i32) {
        (other.x - self.x, other.y - self.y)
    }

    pub fn translate(&mut self, offset: &Position) {
        self.set(self.x + offset.x, self.y + offset.y);
    }

    pub fn get_translated(&self, offset: &Position) -> Position {
        Position::new(self.x + offset.x, self.y + offset.y)
    }

    /// Return distance of this object to a given coordinate.
    pub fn distance(&self, other: &Position) -> f32 {
        (((other.x - self.x).pow(2) + (other.y - self.y).pow(2)) as f32).sqrt()
    }
}
