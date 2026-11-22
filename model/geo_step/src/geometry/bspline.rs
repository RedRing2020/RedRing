//! B-spline の記録を、`geo_nurbs` の型にする
//!
//! STEP のノットは、重複の数とノットの値の組で書かれる。両端を固定していない B-spline（周期的なものを含む）は、
//! ノットの挿入（Boehm の方法）で、定義域 `[u_p, u_{m-p}]` の両端に次数の分だけノットを重ね、同じ形の両端を
//! 固定したノットベクトルに直す（形は変わらない）。有理のものは、重みを掛けた座標（同次座標）で直す。

use crate::record::Point;
use crate::record::{BSplineCurve, BSplineSurface};
use geo_contracts::{NurbsCurve3DConstructor, NurbsSurface3DConstructor};
use geo_core::Point3D;
use geo_nurbs::{NurbsCurve3D, NurbsSurface3D};

/// B-spline の記録を型にできない理由
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum BSplineError {
    /// 制御点の点の記録がない
    MissingPoint,
    /// ノットの数・重複・重みの数が、制御点の数と次数に合わない
    Invalid,
}

/// ノットの値と重複の数から、長さ `expected` のノットベクトルを作る（重複の数の合計が合わなければ、領域を取る
/// 前に `None`）
fn expand(knots: &[f64], multiplicities: &[i64], expected: usize) -> Option<Vec<f64>> {
    if knots.len() != multiplicities.len() {
        return None;
    }
    let total = multiplicities
        .iter()
        .try_fold(0usize, |total, &multiplicity| {
            total.checked_add(usize::try_from(multiplicity).ok()?)
        })?;
    if total != expected {
        return None;
    }
    let mut expanded = Vec::with_capacity(expected);
    for (&knot, &multiplicity) in knots.iter().zip(multiplicities) {
        let multiplicity = usize::try_from(multiplicity)
            .ok()
            .filter(|&count| count >= 1)?;
        expanded.extend(std::iter::repeat_n(knot, multiplicity));
    }
    // ノットは小さい順
    expanded
        .windows(2)
        .all(|pair| matches!(pair, [low, high] if low <= high))
        .then_some(expanded)
}

/// 同次座標の制御点の並び（行ごとに、同じ数の値を持つ）を、ノットベクトル `knots` の両端を固定した形に直す
///
/// 定義域は `[knots[degree], knots[len - 1 - degree]]`。すでに両端が固定されていれば、そのまま返す。
pub(super) fn clamp(
    degree: usize,
    knots: Vec<f64>,
    points: Vec<Vec<f64>>,
) -> Option<(Vec<f64>, Vec<Vec<f64>>)> {
    if knots.len() != points.len() + degree + 1 {
        return None;
    }
    let start = *knots.get(degree)?;
    let end = *knots.get(knots.len() - 1 - degree)?;
    if start >= end {
        return None;
    }
    let (mut knots, mut points) = (knots, points);
    // 始まり: start を次数の分だけ重ね、それより前のノットと制御点を捨てる
    while multiplicity(&knots, start) < degree {
        let span = knots.iter().rposition(|&knot| knot <= start)?;
        (knots, points) = insert(degree, knots, points, start, span)?;
    }
    // start の最後の位置から次数の分だけ前を、新しいノットベクトルの初めにする（start がすでに次数 + 1 個
    // 重なっていても、定義域の外のノットを捨てる）
    let keep_from = knots
        .iter()
        .rposition(|&knot| knot == start)?
        .checked_sub(degree)?;
    knots.drain(..keep_from);
    *knots.first_mut()? = start;
    points.drain(..keep_from);
    // 終わり: end を次数の分だけ重ね、それより後ろのノットと制御点を捨てる
    while multiplicity(&knots, end) < degree {
        // 挿入する値が、すでにノットにあれば、その最後の位置の区間に入れる
        let span = knots.iter().rposition(|&knot| knot <= end)?;
        (knots, points) = insert(degree, knots, points, end, span)?;
    }
    // end の初めの位置から次数の分だけ後ろを、新しいノットベクトルの終わりにする
    let keep_to = knots.iter().position(|&knot| knot == end)? + degree;
    if keep_to >= knots.len() {
        return None;
    }
    knots.truncate(keep_to + 1);
    *knots.last_mut()? = end;
    points.truncate(knots.len() - degree - 1);
    Some((knots, points))
}

/// ノットベクトルの中の `value` の重複の数
fn multiplicity(knots: &[f64], value: f64) -> usize {
    knots.iter().filter(|&&knot| knot == value).count()
}

/// ノット `value`（`knots[span] <= value < knots[span + 1]`）を 1 つ挿入する（Boehm の方法）
fn insert(
    degree: usize,
    knots: Vec<f64>,
    points: Vec<Vec<f64>>,
    value: f64,
    span: usize,
) -> Option<(Vec<f64>, Vec<Vec<f64>>)> {
    let existing = multiplicity(&knots, value);
    let mut inserted = Vec::with_capacity(points.len() + 1);
    for index in 0..=points.len() {
        let point = if index + degree <= span {
            points.get(index)?.clone()
        } else if index + existing <= span {
            let low = *knots.get(index)?;
            let high = *knots.get(index + degree)?;
            let alpha = (value - low) / (high - low);
            let (previous, current) = (points.get(index.checked_sub(1)?)?, points.get(index)?);
            current
                .iter()
                .zip(previous)
                .map(|(current, previous)| alpha * current + (1.0 - alpha) * previous)
                .collect()
        } else {
            points.get(index.checked_sub(1)?)?.clone()
        };
        inserted.push(point);
    }
    let mut knots = knots;
    knots.insert(span + 1, value);
    Some((knots, inserted))
}

/// 点の番号から点を引く関数で、B-spline 曲線の記録を型にする
pub(super) fn curve(
    record: &BSplineCurve,
    point: impl Fn(u64) -> Option<Point>,
) -> Result<NurbsCurve3D<f64>, BSplineError> {
    let degree = usize::try_from(record.degree).map_err(|_| BSplineError::Invalid)?;
    let knots = expand(
        &record.knots,
        &record.multiplicities,
        record.control_points.len() + degree + 1,
    )
    .ok_or(BSplineError::Invalid)?;
    let weights = weights_or_ones(record.weights.clone(), record.control_points.len())?;
    let homogeneous = record
        .control_points
        .iter()
        .zip(&weights)
        .map(|(&id, &weight)| point(id).map(|point| homogeneous(point, weight)))
        .collect::<Option<Vec<_>>>()
        .ok_or(BSplineError::MissingPoint)?;
    let (knots, homogeneous) = clamp(degree, knots, homogeneous).ok_or(BSplineError::Invalid)?;
    let (points, weights): (Vec<Point3D<f64>>, Vec<f64>) =
        homogeneous.iter().map(|values| cartesian(values)).unzip();
    let weights = record.weights.is_some().then_some(weights);
    NurbsCurve3D::new(degree, knots, &points, weights).map_err(|_| BSplineError::Invalid)
}

/// 点の番号から点を引く関数で、B-spline 曲面の記録を型にする
pub(super) fn surface(
    record: &BSplineSurface,
    point: impl Fn(u64) -> Option<Point>,
) -> Result<NurbsSurface3D<f64>, BSplineError> {
    let u_degree = usize::try_from(record.u_degree).map_err(|_| BSplineError::Invalid)?;
    let v_degree = usize::try_from(record.v_degree).map_err(|_| BSplineError::Invalid)?;
    let rows = record.control_points.len();
    let columns = record.control_points.first().map_or(0, Vec::len);
    let u_knots = expand(
        &record.u_knots,
        &record.u_multiplicities,
        rows + u_degree + 1,
    )
    .ok_or(BSplineError::Invalid)?;
    let v_knots = expand(
        &record.v_knots,
        &record.v_multiplicities,
        columns + v_degree + 1,
    )
    .ok_or(BSplineError::Invalid)?;
    if rows == 0 || record.control_points.iter().any(|row| row.len() != columns) {
        return Err(BSplineError::Invalid);
    }
    let weights = match &record.weights {
        Some(weights) => {
            if weights.len() != rows || weights.iter().any(|row| row.len() != columns) {
                return Err(BSplineError::Invalid);
            }
            weights.clone()
        }
        None => vec![vec![1.0; columns]; rows],
    };
    // u の方向の行ごとに、v の方向の制御点の同次座標を並べた値の列
    let mut grid = Vec::with_capacity(rows);
    for (row, row_weights) in record.control_points.iter().zip(&weights) {
        let mut values = Vec::with_capacity(columns * 4);
        for (&id, &weight) in row.iter().zip(row_weights) {
            values.extend(homogeneous(
                point(id).ok_or(BSplineError::MissingPoint)?,
                weight,
            ));
        }
        grid.push(values);
    }
    let (u_knots, grid) = clamp(u_degree, u_knots, grid).ok_or(BSplineError::Invalid)?;
    // v の方向は、行と列を入れ替えて同じように直す
    let transposed = transpose(&grid, columns);
    let (v_knots, transposed) =
        clamp(v_degree, v_knots, transposed).ok_or(BSplineError::Invalid)?;
    let grid = transpose(&transposed, grid.len());
    let mut points = Vec::with_capacity(grid.len());
    let mut weights = Vec::with_capacity(grid.len());
    for values in &grid {
        let (row_points, row_weights): (Vec<Point3D<f64>>, Vec<f64>) =
            values.chunks(4).map(cartesian).unzip();
        points.push(row_points);
        weights.push(row_weights);
    }
    let weights = record.weights.is_some().then_some(weights);
    NurbsSurface3D::new(u_degree, v_degree, u_knots, v_knots, &points, weights)
        .map_err(|_| BSplineError::Invalid)
}

/// 重み（ないときはすべて 1）。数が制御点と合わなければエラー
fn weights_or_ones(weights: Option<Vec<f64>>, count: usize) -> Result<Vec<f64>, BSplineError> {
    match weights {
        Some(weights) if weights.len() == count && weights.iter().all(|&weight| weight > 0.0) => {
            Ok(weights)
        }
        Some(_) => Err(BSplineError::Invalid),
        None => Ok(vec![1.0; count]),
    }
}

/// 点と重みの同次座標 `[w x, w y, w z, w]`
fn homogeneous(point: Point, weight: f64) -> Vec<f64> {
    vec![
        point[0] * weight,
        point[1] * weight,
        point[2] * weight,
        weight,
    ]
}

/// 同次座標の値（4 つ）を、点と重みにする
fn cartesian(values: &[f64]) -> (Point3D<f64>, f64) {
    let value = |index: usize| values.get(index).copied().unwrap_or(1.0);
    let weight = value(3);
    (
        Point3D::new(value(0) / weight, value(1) / weight, value(2) / weight),
        weight,
    )
}

/// 行ごとに `columns` 個の同次座標（4 つずつ）を並べた値の列を、列ごとの並びにする
fn transpose(rows: &[Vec<f64>], columns: usize) -> Vec<Vec<f64>> {
    (0..columns)
        .map(|column| {
            rows.iter()
                .flat_map(|row| row.get(column * 4..column * 4 + 4).unwrap_or_default())
                .copied()
                .collect()
        })
        .collect()
}
