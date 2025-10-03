use crate::geometry::geometry2d::Vector;

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

    /// 右手系で90度回転した法線方向
    pub fn right_hand_normal(&self) -> Self {
        Self::new(-self.y, self.x)
    }

    /// 左手系で90度回転した法線方向（将来拡張用）
    pub fn left_hand_normal(&self) -> Self {
        Self::new(self.y, -self.x)
    }

    /// 方向ベクトルが反時計回り（CCW）か判定
    pub fn is_ccw(&self) -> bool {
        // y成分が正ならCCWとみなす（x軸基準）
        self.y() > 0.0
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

    /// Vector型への変換
    pub fn to_vector(&self) -> Vector {
        Vector::new(self.x, self.y)
    }

    /// 方向ベクトルを反転
    pub fn reverse(&self) -> Self {
        Self::new(-self.x(), -self.y())
    }
}
