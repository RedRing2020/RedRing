//! NURBS変換操作
//!
//! NURBS曲線・曲面に対する変換、挿入、分割などの操作を提供します。

use crate::{KnotVector, Result};
use analysis::Scalar;
use geo_primitives::{Point2D, Point3D};

/// ノット挿入結果の型エイリアス（2D用）
pub type KnotInsertResult2D<T> = Result<(Vec<Point2D<T>>, Vec<T>, KnotVector<T>)>;

/// ノット挿入結果の型エイリアス（3D用）
pub type KnotInsertResult3D<T> = Result<(Vec<Point3D<T>>, Vec<T>, KnotVector<T>)>;

/// 次数上昇結果の型エイリアス（2D用）
pub type DegreeElevateResult2D<T> = Result<(Vec<Point2D<T>>, Vec<T>, KnotVector<T>, usize)>;

/// 次数上昇結果の型エイリアス（3D用）
pub type DegreeElevateResult3D<T> = Result<(Vec<Point3D<T>>, Vec<T>, KnotVector<T>, usize)>;

/// 曲線分割結果の型エイリアス（2D用）
pub type CurveSplitResult2D<T> = Result<(
    (Vec<Point2D<T>>, Vec<T>, KnotVector<T>),
    (Vec<Point2D<T>>, Vec<T>, KnotVector<T>),
)>;

/// 曲線分割結果の型エイリアス（3D用）
pub type CurveSplitResult3D<T> = Result<(
    (Vec<Point3D<T>>, Vec<T>, KnotVector<T>),
    (Vec<Point3D<T>>, Vec<T>, KnotVector<T>),
)>;

/// ノット挿入アルゴリズム
///
/// 指定されたパラメータ値にノットを挿入して、
/// 曲線の形状を変えずに制御点を細分化します。
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
        control_points: &[Point2D<T>],
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

            new_control_points.push(Point2D::new(new_x, new_y));
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
        control_points: &[Point3D<T>],
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

            new_control_points.push(Point3D::new(new_x, new_y, new_z));
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
        control_points: &[Point2D<T>],
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

            new_control_points.push(Point2D::new(new_x, new_y));
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
        control_points: &[Point3D<T>],
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

            new_control_points.push(Point3D::new(new_x, new_y, new_z));
            new_weights.push(new_weight);
        }

        new_control_points.push(control_points[n]);
        new_weights.push(weights[n]);

        Ok((new_control_points, new_weights, new_knots, new_degree))
    }
}

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
        control_points: &[Point2D<T>],
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
        control_points: &[Point3D<T>],
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_knot_insertion_2d() {
        let control_points = vec![
            Point2D::new(0.0, 0.0),
            Point2D::new(1.0, 1.0),
            Point2D::new(2.0, 0.0),
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

    #[test]
    fn test_degree_elevation_2d() {
        let control_points = vec![
            Point2D::new(0.0, 0.0),
            Point2D::new(1.0, 1.0),
            Point2D::new(2.0, 0.0),
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
