/// f64ベース3D点
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Point {
    x: f64,
    y: f64,
    z: f64,
}

impl Point {
    pub fn new(x: f64, y: f64, z: f64) -> Self {
        Self { x, y, z }
    }

    pub fn origin() -> Self {
        Self::new(0.0, 0.0, 0.0)
    }

    pub fn x(&self) -> f64 { self.x }
    pub fn y(&self) -> f64 { self.y }
    pub fn z(&self) -> f64 { self.z }

    pub fn distance_to(&self, other: &Self) -> f64 {
        let dx = self.x - other.x;
        let dy = self.y - other.y;
        let dz = self.z - other.z;
        (dx * dx + dy * dy + dz * dz).sqrt()
    }

    /// 2D点への投影（Z座標を破棄）
    pub fn to_point2d(&self) -> crate::geometry2d::Point2D {
        crate::geometry2d::Point2D::new(self.x, self.y)
    }

    /// XY平面上での距離計算
    pub fn xy_distance_to(&self, other: &Self) -> f64 {
        let dx = self.x - other.x;
        let dy = self.y - other.y;
        (dx * dx + dy * dy).sqrt()
    }
}

impl Default for Point {
    fn default() -> Self {
        Self::origin()
    }
}

// 算術演算の実装
impl std::ops::Add<crate::geometry3d::Vector3D> for Point {
    type Output = Point;

    fn add(self, vec: crate::geometry3d::Vector3D) -> Self::Output {
        Point::new(
            self.x + vec.x(),
            self.y + vec.y(),
            self.z + vec.z()
        )
    }
}

impl std::ops::Sub<Point> for Point {
    type Output = crate::geometry3d::Vector3D;

    fn sub(self, other: Point) -> Self::Output {
        crate::geometry3d::Vector3D::new(
            self.x - other.x,
            self.y - other.y,
            self.z - other.z
        )
    }
}

// Display実装は別クレートで実装

// テストコードはunit_tests/Point_tests.rsに移動
