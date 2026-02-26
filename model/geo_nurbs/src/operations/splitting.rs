//! NURBS曲線分割アルゴリズム
//!
//! 指定されたパラメータ値で曲線を2つの曲線に分割します。
//! 分割後の曲線は元の曲線を正確に表現します。

use crate::{operations::knot_insertion::KnotInsertion, KnotVector, NurbsError, Scalar};
use analysis::linalg::vector::{Vector2, Vector3};

/// 曲線分割結果の型エイリアス（2D用）
pub type CurveSplitResult2D<T> = Result<
    (
        (Vec<Vector2<T>>, Vec<T>, KnotVector<T>),
        (Vec<Vector2<T>>, Vec<T>, KnotVector<T>),
    ),
    NurbsError,
>;

/// 曲線分割結果の型エイリアス（3D用）
pub type CurveSplitResult3D<T> = Result<
    (
        (Vec<Vector3<T>>, Vec<T>, KnotVector<T>),
        (Vec<Vector3<T>>, Vec<T>, KnotVector<T>),
    ),
    NurbsError,
>;

/// 曲線分割アルゴリズム
pub struct CurveSplitting;

impl CurveSplitting {
    /// 指定パラメータで曲線を2つに分割（2D）
    ///
    /// # 引数
    /// * `control_points` - 制御点配列
    /// * `weights` - 重み配列
    /// * `knots` - ノットベクトル
    /// * `degree` - 次数
    /// * `t` - 分割パラメータ
    ///
    /// # 戻り値
    /// ((左側制御点, 左側重み, 左側ノット), (右側制御点, 右側重み, 右側ノット))
    /// NURBS曲線の2D分割
    ///
    /// # Errors
    /// 分割パラメータが範囲外の場合にエラーを返します
    pub fn split_curve_2d<T: Scalar>(
        control_points: &[Vector2<T>],
        weights: &[T],
        knots: &KnotVector<T>,
        degree: usize,
        t: T,
    ) -> CurveSplitResult2D<T> {
        // ノット挿入を degree+1 回実行して完全分割
        let mut current_control_points = control_points.to_vec();
        let mut current_weights = weights.to_vec();
        let mut current_knots = knots.clone();

        for _ in 0..=degree {
            let (new_cp, new_w, new_k) = KnotInsertion::insert_knot_2d(
                &current_control_points,
                &current_weights,
                &current_knots,
                degree,
                t,
            )?;
            current_control_points = new_cp;
            current_weights = new_w;
            current_knots = new_k;
        }

        // 分割点を見つける
        let split_index = crate::knot::find_knot_span(t, &current_knots, degree);

        // 左側曲線
        let left_control_points = current_control_points[..=split_index].to_vec();
        let left_weights = current_weights[..=split_index].to_vec();
        let mut left_knots = current_knots[..=(split_index + degree + 1)].to_vec();

        // 左側ノットベクトルの終端を調整
        for i in (left_knots.len() - degree - 1)..left_knots.len() {
            left_knots[i] = t;
        }

        // 右側曲線
        let right_control_points = current_control_points[split_index..].to_vec();
        let right_weights = current_weights[split_index..].to_vec();
        let mut right_knots = current_knots[split_index..].to_vec();

        // 右側ノットベクトルの開始を調整
        #[allow(clippy::needless_range_loop)] // 曲線分割アルゴリズムの標準実装
        for i in 0..=degree {
            right_knots[i] = t;
        }

        Ok((
            (left_control_points, left_weights, left_knots),
            (right_control_points, right_weights, right_knots),
        ))
    }

    /// 指定パラメータで曲線を2つに分割（3D）
    /// NURBS曲線の3D分割
    ///
    /// # Errors
    /// 分割パラメータが範囲外の場合にエラーを返します
    pub fn split_curve_3d<T: Scalar>(
        control_points: &[Vector3<T>],
        weights: &[T],
        knots: &KnotVector<T>,
        degree: usize,
        t: T,
    ) -> CurveSplitResult3D<T> {
        let mut current_control_points = control_points.to_vec();
        let mut current_weights = weights.to_vec();
        let mut current_knots = knots.clone();

        for _ in 0..=degree {
            let (new_cp, new_w, new_k) = KnotInsertion::insert_knot_3d(
                &current_control_points,
                &current_weights,
                &current_knots,
                degree,
                t,
            )?;
            current_control_points = new_cp;
            current_weights = new_w;
            current_knots = new_k;
        }

        let split_index = crate::knot::find_knot_span(t, &current_knots, degree);

        let left_control_points = current_control_points[..=split_index].to_vec();
        let left_weights = current_weights[..=split_index].to_vec();
        let mut left_knots = current_knots[..=(split_index + degree + 1)].to_vec();

        for i in (left_knots.len() - degree - 1)..left_knots.len() {
            left_knots[i] = t;
        }

        let right_control_points = current_control_points[split_index..].to_vec();
        let right_weights = current_weights[split_index..].to_vec();
        let mut right_knots = current_knots[split_index..].to_vec();

        #[allow(clippy::needless_range_loop)] // 曲線分割アルゴリズムの標準実装
        for i in 0..=degree {
            right_knots[i] = t;
        }

        Ok((
            (left_control_points, left_weights, left_knots),
            (right_control_points, right_weights, right_knots),
        ))
    }
}
