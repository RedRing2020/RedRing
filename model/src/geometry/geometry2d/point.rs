#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Point {
    pub x: f64,
    pub y: f64,
}

impl Point {
    /// 新しい点を生成
    pub fn new(x: f64, y: f64) -> Self {
        Self { x, y }
    }

    /// 他の点との距離を計算
    pub fn distance_to(&self, other: &Point) -> f64 {
        ((self.x - other.x).powi(2) + (self.y - other.y).powi(2)).sqrt()
    }

    /// dx, dy を加算した新しい点を返す
    pub fn add(&self, dx: f64, dy: f64) -> Point {
        Point::new(self.x + dx, self.y + dy)
    }

    /// 他の点との差分ベクトルを返す
    pub fn sub(&self, other: &Point) -> Point {
        Point::new(self.x - other.x, self.y - other.y)
    }

    /// 点を配列形式で取得（互換性用）
    pub fn to_array(&self) -> [f64; 2] {
        [self.x, self.y]
    }

    /// 配列から点を生成
    pub fn from_array(arr: [f64; 2]) -> Self {
        Self { x: arr[0], y: arr[1] }
    }

    /// 線分上の点までの距離を計算
    pub fn distance_to_point_on_line(&self, line: &Line) -> f64 {
        line.distance_to_point(self)
    }
}
