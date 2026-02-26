//! NURBS曲線の次数上昇アルゴリズム
//!
//! 曲線の形状を保ったまま次数を増やす操作を提供します。
//! 次数を上げることで、より滑らかで柔軟な制御が可能になります。

use crate::{KnotVector, NurbsError, Scalar};
use analysis::linalg::vector::{Vector2, Vector3};

/// 次数上昇結果の型エイリアス（2D用）
pub type DegreeElevateResult2D<T> =
    Result<(Vec<Vector2<T>>, Vec<T>, KnotVector<T>, usize), NurbsError>;

/// 次数上昇結果の型エイリアス（3D用）
pub type DegreeElevateResult3D<T> =
    Result<(Vec<Vector3<T>>, Vec<T>, KnotVector<T>, usize), NurbsError>;

/// NURBS曲線の次数上昇
pub struct DegreeElevation;

impl DegreeElevation {
    /// 次数を1つ上昇させる（2D）
    ///
    /// # 引数
    /// * `control_points` - 制御点配列
    /// * `weights` - 重み配列
    /// * `knots` - ノットベクトル
    /// * `degree` - 現在の次数
    ///
    /// # 戻り値
    /// (新しい制御点, 新しい重み, 新しいノットベクトル, 新しい次数)
    /// NURBS曲線の2D次数昇格
    ///
    /// # Errors
    /// 無効な次数や制御点の場合にエラーを返します
    pub fn elevate_degree_2d<T: Scalar>(
        control_points: &[Vector2<T>],
        weights: &[T],
        knots: &KnotVector<T>,
        degree: usize,
    ) -> DegreeElevateResult2D<T> {
        let n = control_points.len() - 1;
        let p = degree;
        let new_degree = p + 1;

        // 新しいノットベクトルを作成（各内部ノットを1回ずつ追加）
        let mut new_knots = Vec::new();

        // 開始ノットを1つ追加
        new_knots.push(knots[0]);
        for &knot in knots {
            new_knots.push(knot);
        }

        // 新しい制御点配列を初期化
        let mut new_control_points = Vec::with_capacity(n + 2);
        let mut new_weights = Vec::with_capacity(n + 2);

        // 最初の制御点はそのまま
        new_control_points.push(control_points[0]);
        new_weights.push(weights[0]);

        // 中間制御点を計算
        for i in 1..=n {
            let alpha = T::from_usize(i) / T::from_usize(new_degree);

            let prev_point = control_points[i - 1];
            let curr_point = control_points[i];
            let prev_weight = weights[i - 1];
            let curr_weight = weights[i];

            let new_x = alpha * prev_point.x() + (T::ONE - alpha) * curr_point.x();
            let new_y = alpha * prev_point.y() + (T::ONE - alpha) * curr_point.y();
            let new_weight = alpha * prev_weight + (T::ONE - alpha) * curr_weight;

            new_control_points.push(Vector2::new(new_x, new_y));
            new_weights.push(new_weight);
        }

        // 最後の制御点を追加
        new_control_points.push(control_points[n]);
        new_weights.push(weights[n]);

        Ok((new_control_points, new_weights, new_knots, new_degree))
    }

    /// 次数を1つ上昇させる（3D）
    /// NURBS曲線の3D次数昇格
    ///
    /// # Errors
    /// 無効な次数や制御点の場合にエラーを返します
    pub fn elevate_degree_3d<T: Scalar>(
        control_points: &[Vector3<T>],
        weights: &[T],
        knots: &KnotVector<T>,
        degree: usize,
    ) -> DegreeElevateResult3D<T> {
        let n = control_points.len() - 1;
        let p = degree;
        let new_degree = p + 1;

        let mut new_knots = Vec::new();
        new_knots.push(knots[0]);
        for &knot in knots {
            new_knots.push(knot);
        }

        let mut new_control_points = Vec::with_capacity(n + 2);
        let mut new_weights = Vec::with_capacity(n + 2);

        new_control_points.push(control_points[0]);
        new_weights.push(weights[0]);

        for i in 1..=n {
            let alpha = T::from_usize(i) / T::from_usize(new_degree);

            let prev_point = control_points[i - 1];
            let curr_point = control_points[i];
            let prev_weight = weights[i - 1];
            let curr_weight = weights[i];

            let new_x = alpha * prev_point.x() + (T::ONE - alpha) * curr_point.x();
            let new_y = alpha * prev_point.y() + (T::ONE - alpha) * curr_point.y();
            let new_z = alpha * prev_point.z() + (T::ONE - alpha) * curr_point.z();
            let new_weight = alpha * prev_weight + (T::ONE - alpha) * curr_weight;

            new_control_points.push(Vector3::new(new_x, new_y, new_z));
            new_weights.push(new_weight);
        }

        new_control_points.push(control_points[n]);
        new_weights.push(weights[n]);

        Ok((new_control_points, new_weights, new_knots, new_degree))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_degree_elevation_2d() {
        let control_points = vec![
            Vector2::new(0.0, 0.0),
            Vector2::new(1.0, 1.0),
            Vector2::new(2.0, 0.0),
        ];
        let weights = vec![1.0, 1.0, 1.0];
        let knots = vec![0.0, 0.0, 0.0, 1.0, 1.0, 1.0];
        let degree = 2;

        let result = DegreeElevation::elevate_degree_2d(&control_points, &weights, &knots, degree);

        assert!(result.is_ok());
        let (new_cp, new_w, new_knots, new_degree) = result.unwrap();

        // 次数上昇後は次数が1つ増える
        assert_eq!(new_degree, degree + 1);
        assert_eq!(new_cp.len(), control_points.len() + 1);
        assert_eq!(new_w.len(), weights.len() + 1);
        assert_eq!(new_knots.len(), knots.len() + 1);
    }
}
