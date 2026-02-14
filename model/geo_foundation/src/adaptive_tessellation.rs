//! NURBS適応的テッセレーション（CPU側パラメータ列生成）

use core::cmp::Ordering;

use crate::Scalar;

/// 適応的テッセレーション設定
#[derive(Debug, Clone, Copy)]
pub struct AdaptiveTessellationSettings<T: Scalar> {
    /// 許容弦誤差
    pub chord_error: T,
    /// 最大分割数
    pub max_subdivisions: u32,
    /// 最小分割数
    pub min_segments: u32,
    /// 最小パラメータ区間幅
    pub min_param_span: T,
}

impl<T: Scalar> AdaptiveTessellationSettings<T> {
    /// 表示トレランスを基準にしたデフォルト設定
    pub fn default_with_tolerance(display_tolerance: T) -> Self {
        Self {
            chord_error: display_tolerance,
            max_subdivisions: 12,
            min_segments: 16,
            min_param_span: display_tolerance,
        }
    }
}

/// 曲線用の適応パラメータ列
#[derive(Debug, Clone)]
pub struct AdaptiveParamList<T: Scalar> {
    /// パラメータ値のリスト（ソート済み、重複なし）
    pub params: Vec<T>,
}

/// サーフェス用の適応パラメータグリッド
#[derive(Debug, Clone)]
pub struct AdaptiveParamGrid<T: Scalar> {
    /// u方向のパラメータ値（ソート済み、重複なし）
    pub u_params: Vec<T>,
    /// v方向のパラメータ値（ソート済み、重複なし）
    pub v_params: Vec<T>,
}

/// 曲線向けの適応分割パラメータ列
pub trait NurbsCurveAdaptiveTessellation<T: Scalar> {
    /// 曲線の弦誤差に基づいて適応的にパラメータ分割を行う
    fn adaptive_params_curve(
        &self,
        settings: &AdaptiveTessellationSettings<T>,
    ) -> AdaptiveParamList<T>;
}

/// サーフェス向けの適応分割パラメータ列
pub trait NurbsSurfaceAdaptiveTessellation<T: Scalar> {
    /// サーフェスの弦誤差（u/v両方向）に基づいて適応的にパラメータ分割を行う
    fn adaptive_params_surface(
        &self,
        settings: &AdaptiveTessellationSettings<T>,
    ) -> AdaptiveParamGrid<T>;
}

/// 1軸の適応パラメータ列を生成
pub fn adaptive_params_axis<T, F>(
    t_min: T,
    t_max: T,
    settings: &AdaptiveTessellationSettings<T>,
    mut error_fn: F,
) -> Vec<T>
where
    T: Scalar,
    F: FnMut(T, T) -> T,
{
    let mut params = vec![t_min, t_max];
    let mut stack = vec![(t_min, t_max, 0u32)];

    while let Some((a, b, depth)) = stack.pop() {
        if b - a <= settings.min_param_span {
            continue;
        }

        let err = error_fn(a, b);
        if err <= settings.chord_error || depth >= settings.max_subdivisions {
            continue;
        }

        let mid = (a + b) / (T::ONE + T::ONE);
        params.push(mid);
        stack.push((a, mid, depth + 1));
        stack.push((mid, b, depth + 1));
    }

    sort_dedup(&mut params);
    ensure_min_segments(&mut params, t_min, t_max, settings.min_segments);
    sort_dedup(&mut params);

    params
}

/// 最小分割数を満たすように均等分割を追加
pub fn ensure_min_segments<T: Scalar>(params: &mut Vec<T>, t_min: T, t_max: T, min_segments: u32) {
    if min_segments == 0 {
        return;
    }

    let target_len = min_segments as usize + 1;
    if params.len() >= target_len {
        return;
    }

    let denom = T::from_usize(min_segments as usize);
    let step = (t_max - t_min) / denom;

    for i in 0..=min_segments as usize {
        let t = t_min + step * T::from_usize(i);
        params.push(t);
    }
}

fn sort_dedup<T: Scalar>(params: &mut Vec<T>) {
    params.sort_by(|a, b| {
        a.to_f64()
            .partial_cmp(&b.to_f64())
            .unwrap_or(Ordering::Equal)
    });
    params.dedup();
}
