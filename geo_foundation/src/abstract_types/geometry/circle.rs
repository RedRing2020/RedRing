//! Circle (円) トレイト定義
//!
//! 2D/3D空間における円の抽象的なインターフェースを提供

use crate::common::constants::{PI, TAU};

/// 2D円の基本操作を定義するトレイト
pub trait Circle2D {
    /// 点の型（通常は Point2D）
    type Point;
    /// ベクトルの型（通常は Vector2D）
    type Vector;

    /// 円の中心座標を取得
    fn center(&self) -> Self::Point;

    /// 円の半径を取得
    fn radius(&self) -> f64;

    /// 円の面積を計算
    fn area(&self) -> f64 {
        PI * self.radius() * self.radius()
    }

    /// 円の周長（円周）を計算
    fn circumference(&self) -> f64 {
        TAU * self.radius()
    }

    /// 円の直径を計算
    fn diameter(&self) -> f64 {
        2.0 * self.radius()
    }

    /// 指定された点が円の内部にあるかを判定
    fn contains_point(&self, point: &Self::Point) -> bool;

    /// 指定された点が円周上にあるかを判定（許容誤差内）
    fn on_circumference(&self, point: &Self::Point, tolerance: f64) -> bool;

    /// 円周上の指定された角度（ラジアン）での点を取得
    /// angle: 0.0 で円の右端（+X軸方向）から反時計回りに測定
    fn point_at_angle(&self, angle: f64) -> Self::Point;

    /// 円周上の指定された点での接線ベクトルを取得
    fn tangent_at_point(&self, point: &Self::Point) -> Option<Self::Vector>;

    /// 円周上の指定された角度での接線ベクトルを取得
    fn tangent_at_angle(&self, angle: f64) -> Self::Vector;

    /// 円の境界ボックス（外接矩形）を取得
    fn bounding_box(&self) -> (Self::Point, Self::Point);
}

/// 3D円の基本操作を定義するトレイト
/// 3D空間における円は平面上に存在する
pub trait Circle3D {
    /// 点の型（通常は Point3D）
    type Point;
    /// ベクトルの型（通常は Vector3D）
    type Vector;

    /// 円の中心座標を取得
    fn center(&self) -> Self::Point;

    /// 円の半径を取得
    fn radius(&self) -> f64;

    /// 円が存在する平面の法線ベクトルを取得
    fn normal(&self) -> Self::Vector;

    /// 円の面積を計算
    fn area(&self) -> f64 {
        PI * self.radius() * self.radius()
    }

    /// 円の周長（円周）を計算
    fn circumference(&self) -> f64 {
        TAU * self.radius()
    }

    /// 円の直径を計算
    fn diameter(&self) -> f64 {
        2.0 * self.radius()
    }

    /// 指定された点が円の内部にあるかを判定（円の平面上で）
    fn contains_point(&self, point: &Self::Point) -> bool;

    /// 指定された点が円周上にあるかを判定（許容誤差内）
    fn on_circumference(&self, point: &Self::Point, tolerance: f64) -> bool;

    /// 円周上の指定された角度（ラジアン）での点を取得
    /// 円の平面内での局所座標系を使用
    fn point_at_angle(&self, angle: f64) -> Self::Point;

    /// 円周上の指定された点での接線ベクトルを取得
    fn tangent_at_point(&self, point: &Self::Point) -> Option<Self::Vector>;

    /// 円周上の指定された角度での接線ベクトルを取得
    fn tangent_at_angle(&self, angle: f64) -> Self::Vector;

    /// 円の境界ボックス（外接直方体）を取得
    fn bounding_box(&self) -> (Self::Point, Self::Point);

    /// 円の平面への投影を2D円として取得
    fn to_2d(&self) -> impl Circle2D;
}

/// 円弧（Arc）の基本操作を定義するトレイト
/// 完全な円の一部分を表現
pub trait Arc2D {
    /// 点の型
    type Point;
    /// ベクトルの型
    type Vector;

    /// 円弧の中心座標を取得
    fn center(&self) -> Self::Point;

    /// 円弧の半径を取得
    fn radius(&self) -> f64;

    /// 円弧の開始角度（ラジアン）を取得
    fn start_angle(&self) -> f64;

    /// 円弧の終了角度（ラジアン）を取得
    fn end_angle(&self) -> f64;

    /// 円弧の角度範囲（ラジアン）を取得
    fn angle_span(&self) -> f64 {
        let mut span = self.end_angle() - self.start_angle();
        if span < 0.0 {
            span += TAU;
        }
        span
    }

    /// 円弧の弧長を計算
    fn arc_length(&self) -> f64 {
        self.radius() * self.angle_span()
    }

    /// 円弧の開始点を取得
    fn start_point(&self) -> Self::Point;

    /// 円弧の終了点を取得
    fn end_point(&self) -> Self::Point;

    /// 円弧上の指定されたパラメータ（0.0〜1.0）での点を取得
    fn point_at_parameter(&self, t: f64) -> Self::Point;

    /// 指定された点が円弧上にあるかを判定
    fn contains_point(&self, point: &Self::Point, tolerance: f64) -> bool;
}

/// 3D円弧のトレイト
pub trait Arc3D {
    /// 点の型
    type Point;
    /// ベクトルの型
    type Vector;

    /// 円弧の中心座標を取得
    fn center(&self) -> Self::Point;

    /// 円弧の半径を取得
    fn radius(&self) -> f64;

    /// 円弧が存在する平面の法線ベクトルを取得
    fn normal(&self) -> Self::Vector;

    /// 円弧の開始角度（ラジアン）を取得
    fn start_angle(&self) -> f64;

    /// 円弧の終了角度（ラジアン）を取得
    fn end_angle(&self) -> f64;

    /// 円弧の角度範囲（ラジアン）を取得
    fn angle_span(&self) -> f64 {
        let mut span = self.end_angle() - self.start_angle();
        if span < 0.0 {
            span += TAU;
        }
        span
    }

    /// 円弧の弧長を計算
    fn arc_length(&self) -> f64 {
        self.radius() * self.angle_span()
    }

    /// 円弧の開始点を取得
    fn start_point(&self) -> Self::Point;

    /// 円弧の終了点を取得
    fn end_point(&self) -> Self::Point;

    /// 円弧上の指定されたパラメータ（0.0〜1.0）での点を取得
    fn point_at_parameter(&self, t: f64) -> Self::Point;

    /// 指定された点が円弧上にあるかを判定
    fn contains_point(&self, point: &Self::Point, tolerance: f64) -> bool;

    /// 円弧の平面への投影を2D円弧として取得
    fn to_2d(&self) -> impl Arc2D;
}
