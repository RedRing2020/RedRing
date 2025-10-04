//! Direction2D - 正規化済み 2D 方向ベクトル
//!
//! 以前 geometry2d モジュールを縮小していた段階では未公開だったが、
//! line / ray / infinite_line の再公開に伴い最小実装として復活。

use geo_core::{Vector2D, ToleranceContext, TolerantEq};

#[derive(Debug, Clone)]
pub struct Direction2D { x: f64, y: f64 }

impl Direction2D {
    pub fn from_f64(x: f64, y: f64) -> Option<Self> {
        let m = (x*x + y*y).sqrt();
        if m > 1e-12 { Some(Self { x: x/m, y: y/m }) } else { None }
    }
    pub fn from_vector(v: &Vector2D) -> Option<Self> { Self::from_f64(v.x(), v.y()) }
    pub fn unit_x() -> Self { Self { x:1.0, y:0.0 } }
    pub fn unit_y() -> Self { Self { x:0.0, y:1.0 } }
    pub fn x(&self) -> f64 { self.x }
    pub fn y(&self) -> f64 { self.y }
    pub fn to_vector(&self) -> Vector2D { Vector2D::new(self.x, self.y) }
    pub fn negate(&self) -> Self { Self { x:-self.x, y:-self.y } }
    pub fn rotate(&self, angle: f64) -> Self {
        let (s,c)=angle.sin_cos();
        Self { x: self.x * c - self.y * s, y: self.x * s + self.y * c }
    }
}

impl TolerantEq for Direction2D {
    fn tolerant_eq(&self, other: &Self, ctx: &ToleranceContext) -> bool {
        (self.x - other.x).abs() <= ctx.angular && (self.y - other.y).abs() <= ctx.angular
    }
}
