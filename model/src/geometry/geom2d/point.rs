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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_distance() {
        let p1 = Point::new(0.0, 0.0);
        let p2 = Point::new(3.0, 4.0);
        assert_eq!(p1.distance_to(&p2), 5.0);
    }

    #[test]
    fn test_add() {
        let p = Point::new(1.0, 2.0);
        let result = p.add(3.0, -1.0);
        assert_eq!(result, Point::new(4.0, 1.0));
    }

    #[test]
    fn test_sub() {
        let p1 = Point::new(5.0, 3.0);
        let p2 = Point::new(2.0, 1.0);
        let result = p1.sub(&p2);
        assert_eq!(result, Point::new(3.0, 2.0));
    }
}