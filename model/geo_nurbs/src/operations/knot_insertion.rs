//! ノット挿入アルゴリズム
//!
//! 指定されたパラメータ値にノットを挿入して、
//! 曲線の形状を変えずに制御点を細分化します。

use crate::{KnotVector, NurbsError, Scalar};
use analysis::linalg::vector::{Vector2, Vector3};

/// ノット挿入結果の型エイリアス（2D用）
pub type KnotInsertResult2D<T> = Result<(Vec<Vector2<T>>, Vec<T>, KnotVector<T>), NurbsError>;

/// ノット挿入結果の型エイリアス（3D用）
pub type KnotInsertResult3D<T> = Result<(Vec<Vector3<T>>, Vec<T>, KnotVector<T>), NurbsError>;

/// ノット挿入アルゴリズム
pub struct KnotInsertion;

impl KnotInsertion {
    /// ノット挿入（1回）
    ///
    /// # 引数
    /// * `control_points` - 元の制御点配列
    /// * `weights` - 元の重み配列
    /// * `knots` - 元のノットベクトル
    /// * `degree` - 次数
    /// * `u` - 挿入するパラメータ値
    ///
    /// # 戻り値
    /// (新しい制御点, 新しい重み, 新しいノットベクトル)
    /// NURBS曲線の2Dノット挿入
    ///
    /// # Errors
    /// 無効なノットベクトルや次数の場合にエラーを返します
    #[allow(clippy::many_single_char_names)] // 数学記号は標準的
    pub fn insert_knot_2d<T: Scalar>(
        control_points: &[Vector2<T>],
        weights: &[T],
        knots: &KnotVector<T>,
        degree: usize,
        u: T,
    ) -> KnotInsertResult2D<T> {
        let n = control_points.len() - 1; // 制御点数 - 1
        let p = degree;
        let m = knots.len() - 1; // ノット数 - 1

        // 挿入位置を見つける
        let k = crate::knot::find_knot_span(u, knots, degree);

        // 新しい配列を準備
        let mut new_control_points = Vec::with_capacity(control_points.len() + 1);
        let mut new_weights = Vec::with_capacity(weights.len() + 1);
        let mut new_knots = Vec::with_capacity(knots.len() + 1);

        // 新しいノットベクトルを構築
        #[allow(clippy::needless_range_loop)] // ノット挿入アルゴリズムの標準実装
        for i in 0..=k {
            new_knots.push(knots[i]);
        }
        new_knots.push(u);
        #[allow(clippy::needless_range_loop)] // ノット挿入アルゴリズムの標準実装
        for i in (k + 1)..=m {
            new_knots.push(knots[i]);
        }

        // 制御点と重みを更新
        // 影響を受けない前半部分
        for i in 0..=(k - p) {
            new_control_points.push(control_points[i]);
            new_weights.push(weights[i]);
        }

        // ノット挿入による影響範囲
        for i in (k - p + 1)..=k {
            let alpha = (u - knots[i]) / (knots[i + p] - knots[i]);

            let old_point = control_points[i];
            let prev_point = control_points[i - 1];
            let old_weight = weights[i];
            let prev_weight = weights[i - 1];

            let new_x = (T::ONE - alpha) * prev_point.x() + alpha * old_point.x();
            let new_y = (T::ONE - alpha) * prev_point.y() + alpha * old_point.y();
            let new_weight = (T::ONE - alpha) * prev_weight + alpha * old_weight;

            new_control_points.push(Vector2::new(new_x, new_y));
            new_weights.push(new_weight);
        }

        // 影響を受けない後半部分
        for i in k..=n {
            new_control_points.push(control_points[i]);
            new_weights.push(weights[i]);
        }

        Ok((new_control_points, new_weights, new_knots))
    }

    /// ノット挿入（3D版）
    /// NURBS曲線の3Dノット挿入
    ///
    /// # Errors
    /// 無効なノットベクトルや次数の場合にエラーを返します
    #[allow(clippy::many_single_char_names)] // 数学記号は標準的
    pub fn insert_knot_3d<T: Scalar>(
        control_points: &[Vector3<T>],
        weights: &[T],
        knots: &KnotVector<T>,
        degree: usize,
        u: T,
    ) -> KnotInsertResult3D<T> {
        let n = control_points.len() - 1;
        let p = degree;
        let m = knots.len() - 1;

        let k = crate::knot::find_knot_span(u, knots, degree);

        let mut new_control_points = Vec::with_capacity(control_points.len() + 1);
        let mut new_weights = Vec::with_capacity(weights.len() + 1);
        let mut new_knots = Vec::with_capacity(knots.len() + 1);

        // 新しいノットベクトル
        #[allow(clippy::needless_range_loop)] // ノット挿入アルゴリズムの標準実装
        for i in 0..=k {
            new_knots.push(knots[i]);
        }
        new_knots.push(u);
        #[allow(clippy::needless_range_loop)] // ノット挿入アルゴリズムの標準実装
        for i in (k + 1)..=m {
            new_knots.push(knots[i]);
        }

        // 前半部分
        for i in 0..=(k - p) {
            new_control_points.push(control_points[i]);
            new_weights.push(weights[i]);
        }

        // 影響範囲
        for i in (k - p + 1)..=k {
            let alpha = (u - knots[i]) / (knots[i + p] - knots[i]);

            let old_point = control_points[i];
            let prev_point = control_points[i - 1];
            let old_weight = weights[i];
            let prev_weight = weights[i - 1];

            let new_x = (T::ONE - alpha) * prev_point.x() + alpha * old_point.x();
            let new_y = (T::ONE - alpha) * prev_point.y() + alpha * old_point.y();
            let new_z = (T::ONE - alpha) * prev_point.z() + alpha * old_point.z();
            let new_weight = (T::ONE - alpha) * prev_weight + alpha * old_weight;

            new_control_points.push(Vector3::new(new_x, new_y, new_z));
            new_weights.push(new_weight);
        }

        // 後半部分
        for i in k..=n {
            new_control_points.push(control_points[i]);
            new_weights.push(weights[i]);
        }

        Ok((new_control_points, new_weights, new_knots))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_knot_insertion_2d() {
        let control_points = vec![
            Vector2::new(0.0, 0.0),
            Vector2::new(1.0, 1.0),
            Vector2::new(2.0, 0.0),
        ];
        let weights = vec![1.0, 1.0, 1.0];
        let knots = vec![0.0, 0.0, 0.0, 1.0, 1.0, 1.0];
        let degree = 2;

        let result = KnotInsertion::insert_knot_2d(&control_points, &weights, &knots, degree, 0.5);

        assert!(result.is_ok());
        let (new_cp, new_w, new_knots) = result.unwrap();

        // ノット挿入後は制御点が1つ増える
        assert_eq!(new_cp.len(), control_points.len() + 1);
        assert_eq!(new_w.len(), weights.len() + 1);
        assert_eq!(new_knots.len(), knots.len() + 1);
    }
}
