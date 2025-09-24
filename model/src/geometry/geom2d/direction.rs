#[derive(Debug, Clone, PartialEq)]
pub struct Direction {
    x: f64,
    y: f64,
}

impl Direction {
    /// 正規化された方向ベクトルを生成
    pub fn new(x: f64, y: f64) -> Self {
        let mag = (x.powi(2) + y.powi(2)).sqrt();
        if mag == 0.0 {
            Self { x: 0.0, y: 0.0 }
        } else {
            Self { x: x / mag, y: y / mag }
        }
    }

    /// x成分を取得
    pub fn x(&self) -> f64 {
        self.x
    }

    /// y成分を取得
    pub fn y(&self) -> f64 {
        self.y
    }

    /// 配列形式で取得
    pub fn to_array(&self) -> [f64; 2] {
        [self.x, self.y]
    }

    /// 法線方向を取得（右手系90度回転）
    pub fn normal(&self) -> Self {
        Self::new(-self.y, self.x)
    }

    /// 他の方向との内積
    pub fn dot(&self, other: &Self) -> f64 {
        self.x * other.x + self.y * other.y
    }

    /// 他の方向との角度（ラジアン）
    pub fn angle_to(&self, other: &Self) -> f64 {
        let dot = self.dot(other);
        dot.clamp(-1.0, 1.0).acos()
    }

    /// 等価判定（将来的に誤差許容付き比較に拡張可能）
    pub fn approx_eq(&self, other: &Self, epsilon: f64) -> bool {
        (self.x - other.x).abs() < epsilon && (self.y - other.y).abs() < epsilon
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normalization() {
        let dir = Direction::new(3.0, 4.0);
        let mag = (dir.x.powi(2) + dir.y.powi(2)).sqrt();
        assert!((mag - 1.0).abs() < 1e-10);
        assert!((dir.x - 0.6).abs() < 1e-10);
        assert!((dir.y - 0.8).abs() < 1e-10);
    }

    #[test]
    fn test_zero_vector() {
        let dir = Direction::new(0.0, 0.0);
        assert_eq!(dir.x, 0.0);
        assert_eq!(dir.y, 0.0);
    }

    #[test]
    fn test_to_array() {
        let dir = Direction::new(1.0, 0.0);
        assert_eq!(dir.to_array(), [1.0, 0.0]);
    }

    #[test]
    fn test_normal_vector() {
        let dir = Direction::new(1.0, 0.0);
        let normal = dir.normal();
        assert_eq!(normal, Direction::new(0.0, 1.0));
    }

    #[test]
    fn test_dot_product() {
        let a = Direction::new(1.0, 0.0);
        let b = Direction::new(0.0, 1.0);
        assert_eq!(a.dot(&b), 0.0);

        let c = Direction::new(1.0, 0.0);
        let d = Direction::new(1.0, 0.0);
        assert!((c.dot(&d) - 1.0).abs() < 1e-10);
    }

    #[test]
    fn test_angle_to() {
        let a = Direction::new(1.0, 0.0);
        let b = Direction::new(0.0, 1.0);
        let angle = a.angle_to(&b);
        assert!((angle - std::f64::consts::FRAC_PI_2).abs() < 1e-10);
    }
}