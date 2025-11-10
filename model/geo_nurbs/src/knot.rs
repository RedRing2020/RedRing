//! ノットベクトル操作とバリデーション

use crate::{NurbsError, Result};
use geo_foundation::Scalar;

/// ノットベクトルの型エイリアス
pub type KnotVector<T> = Vec<T>;

/// ノットベクトルの有効性を検証
///
/// # Errors
/// * ノットベクトルが非単調の場合
/// * 長さが不正な場合
/// * 重複度が不正な場合
pub fn validate_knot_vector<T: Scalar>(
    knots: &[T],
    degree: usize,
    num_control_points: usize,
) -> Result<()> {
    // 必要なノット数を計算: n + p + 1 (n=制御点数-1, p=次数)
    let required_length = num_control_points + degree + 1;

    if knots.len() != required_length {
        return Err(NurbsError::invalid_knot_vector(format!(
            "ノットベクトルの長さが不正: {}個. {}個必要です",
            knots.len(),
            required_length
        )));
    }

    // 単調性をチェック
    for i in 1..knots.len() {
        if knots[i] < knots[i - 1] {
            return Err(NurbsError::invalid_knot_vector(format!(
                "ノットベクトルが非単調: knots[{}]={:?} < knots[{}]={:?}",
                i,
                knots[i],
                i - 1,
                knots[i - 1]
            )));
        }
    }

    // 最初と最後の重複度をチェック（度数+1回重複する必要がある）
    let start_multiplicity = count_multiplicity_at_start(knots);
    let end_multiplicity = count_multiplicity_at_end(knots);

    if start_multiplicity < degree + 1 {
        return Err(NurbsError::invalid_knot_vector(format!(
            "開始ノットの重複度が不足: {}回. {}回以上必要です",
            start_multiplicity,
            degree + 1
        )));
    }

    if end_multiplicity < degree + 1 {
        return Err(NurbsError::invalid_knot_vector(format!(
            "終端ノットの重複度が不足: {}回. {}回以上必要です",
            end_multiplicity,
            degree + 1
        )));
    }

    Ok(())
}

/// 均等ノットベクトルを生成
///
/// # 引数
/// * `degree` - NURBS次数
/// * `num_control_points` - 制御点数
/// * `start` - 開始パラメータ
/// * `end` - 終了パラメータ
///
/// # 戻り値
/// 均等間隔のノットベクトル
pub fn create_uniform_knot_vector<T: Scalar>(
    degree: usize,
    num_control_points: usize,
    start: T,
    end: T,
) -> KnotVector<T> {
    let mut knots = Vec::new();
    let total_knots = num_control_points + degree + 1;

    // 開始部分: degree+1個の同じ値
    for _ in 0..=degree {
        knots.push(start);
    }

    // 中央部分: 均等間隔
    let internal_knots = total_knots - 2 * (degree + 1);
    if internal_knots > 0 {
        let divisor = T::from_usize(internal_knots + 1);
        let interval = (end - start) / divisor;
        for i in 1..=internal_knots {
            let multiplier = T::from_usize(i);
            knots.push(start + interval * multiplier);
        }
    }

    // 終了部分: degree+1個の同じ値
    for _ in 0..=degree {
        knots.push(end);
    }

    knots
}

/// クランプノットベクトルを生成（開始・終了で完全に重複）
///
/// # 引数
/// * `degree` - NURBS次数
/// * `num_control_points` - 制御点数
///
/// # 戻り値
/// [0,1]区間のクランプノットベクトル
pub fn create_clamped_knot_vector<T: Scalar>(
    degree: usize,
    num_control_points: usize,
) -> KnotVector<T> {
    create_uniform_knot_vector(degree, num_control_points, T::ZERO, T::ONE)
}

/// オープンノットベクトルを生成（内部に重複なし）
///
/// # 引数
/// * `degree` - NURBS次数
/// * `num_control_points` - 制御点数
///
/// # 戻り値
/// 内部重複のないノットベクトル
#[must_use]
pub fn create_open_knot_vector<T: Scalar>(
    degree: usize,
    num_control_points: usize,
) -> KnotVector<T> {
    let mut knots = Vec::new();
    let total_knots = num_control_points + degree + 1;

    for i in 0..total_knots {
        let t = T::from_usize(i) / T::from_usize(total_knots - 1);
        knots.push(t);
    }

    knots
}

/// ノットベクトルの定義域を取得
///
/// # 引数
/// * `knots` - ノットベクトル
/// * `degree` - NURBS次数
///
/// # 戻り値
/// (開始パラメータ, 終了パラメータ)
pub fn get_parameter_domain<T: Scalar>(knots: &[T], degree: usize) -> (T, T) {
    let start = knots[degree];
    let end = knots[knots.len() - degree - 1];
    (start, end)
}

/// 指定パラメータでのノットスパンを検索
///
/// # 引数
/// * `t` - パラメータ値
/// * `knots` - ノットベクトル
/// * `degree` - NURBS次数
///
/// # 戻り値
/// ノットスパンインデックス
pub fn find_knot_span<T: Scalar>(t: T, knots: &[T], degree: usize) -> usize {
    let n = knots.len() - degree - 1;

    // 境界値の処理
    if t >= knots[n] {
        return n - 1;
    }

    if t <= knots[degree] {
        return degree;
    }

    // バイナリサーチ
    let mut low = degree;
    let mut high = n;

    while low < high {
        let mid = usize::midpoint(low, high);
        if t < knots[mid] {
            high = mid;
        } else {
            low = mid + 1;
        }
    }

    low - 1
}

/// 開始位置での重複度を数える
fn count_multiplicity_at_start<T: Scalar>(knots: &[T]) -> usize {
    if knots.is_empty() {
        return 0;
    }

    let first_knot = knots[0];
    knots.iter().take_while(|&&knot| knot == first_knot).count()
}

/// 終端位置での重複度を数える
fn count_multiplicity_at_end<T: Scalar>(knots: &[T]) -> usize {
    if knots.is_empty() {
        return 0;
    }

    let last_knot = knots[knots.len() - 1];
    knots
        .iter()
        .rev()
        .take_while(|&&knot| knot == last_knot)
        .count()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_uniform_knot_vector() {
        let knots = create_uniform_knot_vector(2, 4, 0.0, 1.0);
        assert_eq!(knots, vec![0.0, 0.0, 0.0, 0.5, 1.0, 1.0, 1.0]);
    }

    #[test]
    fn test_clamped_knot_vector() {
        let knots = create_clamped_knot_vector::<f64>(1, 3);
        assert_eq!(knots, vec![0.0, 0.0, 0.5, 1.0, 1.0]);
    }

    #[test]
    fn test_knot_validation() {
        let knots = vec![0.0, 0.0, 0.5, 1.0, 1.0];
        assert!(validate_knot_vector(&knots, 1, 3).is_ok());

        // 非単調
        let bad_knots = vec![0.0, 1.0, 0.5, 1.0, 1.0];
        assert!(validate_knot_vector(&bad_knots, 1, 3).is_err());
    }

    #[test]
    fn test_find_knot_span() {
        let knots = vec![0.0, 0.0, 0.0, 1.0, 1.0, 1.0];
        assert_eq!(find_knot_span(0.0, &knots, 2), 2);
        assert_eq!(find_knot_span(0.5, &knots, 2), 2);
        assert_eq!(find_knot_span(1.0, &knots, 2), 2);
    }

    #[test]
    fn test_parameter_domain() {
        let knots = vec![0.0, 0.0, 0.0, 1.0, 1.0, 1.0];
        let (start, end) = get_parameter_domain(&knots, 2);
        assert!((start - 0.0).abs() < f64::EPSILON);
        assert!((end - 1.0).abs() < f64::EPSILON);
    }
}
