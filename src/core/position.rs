#[derive(Debug, Serialize, Deserialize, Default)]
pub struct Position {
    pub x: i32,
    pub y: i32,
}

impl Position {
    pub fn new(x: i32, y: i32) -> Self {
        Position { x, y }
    }

    pub fn is_equal(&self, other: &Position) -> bool {
        self.x == other.x && self.y == other.y
    }

    pub fn is_eq(&self, x: i32, y: i32) -> bool {
        self.x == x && self.y == y
    }

    pub fn set(&mut self, a: i32, b: i32) {
        self.x = a;
        self.y = b;
    }

    pub fn is_adjacent(&self, other: &Position) -> bool {
        (other.x - self.x).abs() <= 1
            && (other.y - self.y).abs() <= 1
            && ((other.x - self.x) - (other.y - self.y)).abs() == 1
    }

    pub fn offset(&self, other: &Position) -> (i32, i32) {
        (other.x - self.x, other.y - self.y)
    }

    pub fn translate(&mut self, offset: &Position) {
        self.x += offset.x;
        self.y += offset.y;
    }

    pub fn get_translated(&self, offset: &Position) -> Position {
        Position::new(self.x + offset.x, self.y + offset.y)
    }

    /// Return distance of this object to a given coordinate.
    pub fn distance(&self, other: &Position) -> f32 {
        (((other.x - self.x).pow(2) + (other.y - self.y).pow(2)) as f32).sqrt()
    }
}
