use crate::geometry_trait::Normalize;

use super::vector::Vector;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Direction(Vector);

impl Direction {
    /// 安全に正規化されたDirectionを生成、零ベクトルの場合はNoneを返す。
    /// 成分からDirectionを作成する推奨方法。
    pub fn new(x: f64, y: f64, z: f64) -> Option<Self> {
        let v = Vector::new(x, y, z);
        Self::from_vector(v)
    }

    /// 正規化チェックなしでDirectionを作成（不安全、呼び出し側が単位長を保証必要）。
    /// 入力が既に正規化済みであることが確実な場合のみ使用。
    /// 外部で正規化が保証されているパフォーマンス重視コード用。
    pub fn new_unchecked(x: f64, y: f64, z: f64) -> Self {
        Direction(Vector::new(x, y, z))
    }

    pub fn from_vector(v: Vector) -> Option<Self> {
        // 元のベクトルが零ベクトルかチェック
        if v.norm() == 0.0 {
            None
        } else {
            let direction = Direction(v.normalize());
            // 念のため、作成された Direction の長さをチェック
            if direction.length() > 0.0 {
                Some(direction)
            } else {
                None
            }
        }
    }

    pub fn length(&self) -> f64 {
        self.0.norm()
    }

    pub fn x(&self) -> f64 {
        self.0.x()
    }

    pub fn y(&self) -> f64 {
        self.0.y()
    }

    pub fn z(&self) -> f64 {
        self.0.z()
    }

    pub fn normalize(&self) -> Option<Self> {
        // Direction は既に正規化されたベクトルなので、selfを返す
        // ただし、数値誤差により長さが1でない可能性があるため、再正規化
        Direction::from_vector(self.0)
    }

    pub fn as_vector(&self) -> Vector {
        self.0
    }

    pub fn to_vector(&self) -> Vector {
        self.0
    }

    pub fn dot(&self, other: &Self) -> f64 {
        self.0.dot(&other.0)
    }

    pub fn cross(&self, other: &Self) -> Option<Self> {
        let cross = self.0.cross(&other.0);
        Direction::from_vector(cross)
    }

    pub fn negate(&self) -> Self {
        Direction(Vector::new(-self.0.x(), -self.0.y(), -self.0.z()))
    }

    /// normalベクトルから円の直交基底(u, v)を生成
    pub fn orthonormal_basis(&self) -> (Vector, Vector) {
        let n = self.to_vector();
        // nと直交する任意のベクトルを選ぶ
        let up = if n.z().abs() < 0.99 {
            Vector::new(0.0, 0.0, 1.0)
        } else {
            Vector::new(0.0, 1.0, 0.0)
        };
        let u = n.cross(&up).normalize();
        let v = n.cross(&u).normalize();
        (u, v)
    }
}
