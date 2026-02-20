//! NURBS GPU評価用のデータ変換
//!
//! Model層（geo_nurbs）のNURBS形状とパラメータリストを、
//! View層（render）でGPU評価可能な形式に変換します。

use geo_algorithms::adaptive_tessellation;
use geo_foundation::Scalar;

/// NURBS曲線のGPU評価に必要なデータ
///
/// CPU側で生成した適応パラメータリストと、NURBS曲線定義を
/// GPU（WGSL）で処理可能なf32配列に変換したデータ。
#[derive(Debug, Clone)]
pub struct NurbsCurveEvalData {
    /// 評価パラメータ列（ソート済み、adaptive tessellation結果）
    pub params: Vec<f32>,

    /// 制御点（flattenされた配列: [x0, y0, z0, x1, y1, z1, ...]）
    pub control_points: Vec<f32>,

    /// 重み（有理NURBS用、Noneなら非有理として全て1.0のダミーを使用）
    pub weights: Option<Vec<f32>>,

    /// ノットベクトル
    pub knots: Vec<f32>,

    /// 次数
    pub degree: u32,
}

impl NurbsCurveEvalData {
    /// NurbsCurve3DとAdaptiveParamListから生成
    ///
    /// # Arguments
    /// * `curve` - NURBS曲線（任意のScalar型T）
    /// * `param_list` - CPU側適応分割結果のパラメータリスト
    ///
    /// # Returns
    /// GPU評価用のf32変換済みデータ
    pub fn from_curve_params<T: Scalar>(
        curve: &impl geo_foundation::NurbsCurve3DProperties<T>,
        param_list: &adaptive_tessellation::AdaptiveParamList<T>,
    ) -> Self {
        let degree = curve.degree() as u32;

        // ノットベクトルの変換
        let knots: Vec<f32> = curve.knot_vector().iter().map(|k| k.to_f32()).collect();

        // 制御点のflatten: coordinates()で直接フラット配列を取得
        let num_cp = curve.control_points_count();
        let control_points: Vec<f32> = curve.coordinates().iter().map(|c| c.to_f32()).collect();

        // 重みの変換（Foundationトレイトメソッド使用）
        let weights = curve
            .weights()
            .map(|w_slice| w_slice.iter().map(|wi| wi.to_f32()).collect());

        // パラメータの変換（Vec<T> -> Vec<f32>）
        let params: Vec<f32> = param_list.params.iter().map(|p| p.to_f32()).collect();

        tracing::debug!(
            "📋 NurbsCurveEvalData 変換完了: {} params, {} control points, degree={}, {} knots",
            params.len(),
            num_cp,
            degree,
            knots.len()
        );

        if !control_points.is_empty() {
            tracing::trace!(
                "📋 制御点[0]: ({:.3}, {:.3}, {:.3})",
                control_points[0],
                control_points[1],
                control_points[2]
            );
            if num_cp > 1 {
                let last_idx = control_points.len() - 3;
                tracing::trace!(
                    "📋 制御点[{}]: ({:.3}, {:.3}, {:.3})",
                    num_cp - 1,
                    control_points[last_idx],
                    control_points[last_idx + 1],
                    control_points[last_idx + 2]
                );
            }
        }

        Self {
            params,
            control_points,
            weights,
            knots,
            degree,
        }
    }

    /// 制御点数を取得
    pub fn num_control_points(&self) -> usize {
        self.control_points.len() / 3
    }

    /// 評価点数を取得（LineStrip頂点数）
    pub fn num_eval_points(&self) -> usize {
        self.params.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use geo_foundation::NurbsCurve3DConstructor;
    use geo_nurbs::{
        adaptive_tessellation::{AdaptiveTessellationSettings, NurbsCurveAdaptiveTessellation},
        NurbsCurve3D,
    };

    #[test]
    fn test_nurbs_curve_eval_data_from_line() {
        // 直線のNURBS曲線を作成（(0,0,0) -> (1,0,0)）
        let curve = NurbsCurve3D::<f64>::line_segment((0.0, 0.0, 0.0), (1.0, 0.0, 0.0)).unwrap();

        // 適応パラメータ生成
        let settings = AdaptiveTessellationSettings::default_with_tolerance(0.01);
        let param_list = curve.adaptive_params_curve(&settings);

        // geo_foundation の型に変換
        let param_list_foundation = adaptive_tessellation::AdaptiveParamList {
            params: param_list.params.clone(),
        };

        // GPU評価用データに変換
        let eval_data = NurbsCurveEvalData::from_curve_params(&curve, &param_list_foundation);

        // 検証
        assert_eq!(eval_data.degree, 1);
        assert_eq!(eval_data.num_control_points(), 2);
        assert!(eval_data.num_eval_points() >= 2); // 最低2点（始点・終点）

        // 制御点のflatten確認
        assert_eq!(eval_data.control_points.len(), 6); // 2点 * 3座標
        assert_eq!(eval_data.control_points[0], 0.0);
        assert_eq!(eval_data.control_points[1], 0.0);
        assert_eq!(eval_data.control_points[2], 0.0);
        assert_eq!(eval_data.control_points[3], 1.0);
        assert_eq!(eval_data.control_points[4], 0.0);
        assert_eq!(eval_data.control_points[5], 0.0);

        // パラメータ確認（始点0.0、終点1.0が含まれる）
        assert_eq!(eval_data.params.first(), Some(&0.0));
        assert_eq!(eval_data.params.last(), Some(&1.0));

        // 非有理曲線なので重みはNone
        assert!(eval_data.weights.is_none());
    }
}

/// NURBS曲面のGPU評価に必要なデータ
///
/// CPU側で生成した適応パラメータグリッドと、NURBS曲面定義を
/// GPU（WGSL）で処理可能なf32配列に変換したデータ。
///
/// # メモリレイアウト最適化
/// - 頂点パラメータ: インターリーブド配列 `[u0,v0, u1,v1, ...]`
/// - 制御点: フラット配列 `[x0,y0,z0, x1,y1,z1, ...]`
/// - Storage Buffer数を最小化
#[derive(Debug, Clone)]
pub struct NurbsSurfaceEvalData {
    /// 頂点パラメータ（インターリーブド: [u0,v0, u1,v1, ...]）
    /// 各頂点の(u,v)評価パラメータをグリッド順に格納
    pub vertex_params: Vec<f32>,

    /// 制御点グリッド（flattenされた配列: [x0,y0,z0, x1,y1,z1, ...]、u方向優先）
    pub control_points: Vec<f32>,

    /// 重み（有理NURBS用、Noneなら非有理として全て1.0のダミーを使用）
    pub weights: Option<Vec<f32>>,

    /// u方向ノットベクトル
    pub u_knots: Vec<f32>,

    /// v方向ノットベクトル
    pub v_knots: Vec<f32>,

    /// u方向次数
    pub u_degree: u32,

    /// v方向次数
    pub v_degree: u32,

    /// グリッドサイズ
    pub u_count: u32,
    pub v_count: u32,

    /// 評価グリッドサイズ（頂点数計算用）
    num_u_params: usize,
    num_v_params: usize,
}

impl NurbsSurfaceEvalData {
    /// NurbsSurface3DとAdaptiveParamGridから生成
    ///
    /// # Arguments
    /// * `surface` - NURBS曲面（任意のScalar型T）
    /// * `param_grid` - CPU側適応分割結果のパラメータグリッド
    ///
    /// # Returns
    /// GPU評価用のf32変換済みデータ（頂点バッファ最適化）
    pub fn from_surface_params<T: Scalar>(
        surface: &impl geo_foundation::NurbsSurface3DProperties<T>,
        param_grid: &adaptive_tessellation::AdaptiveParamGrid<T>,
    ) -> Self {
        let u_degree = surface.u_degree() as u32;
        let v_degree = surface.v_degree() as u32;
        let u_count = surface.u_count() as u32;
        let v_count = surface.v_count() as u32;

        // u, v パラメータの変換（一時的）
        let u_params: Vec<f32> = param_grid.u_params.iter().map(|p| p.to_f32()).collect();
        let v_params: Vec<f32> = param_grid.v_params.iter().map(|p| p.to_f32()).collect();

        let num_u_params = u_params.len();
        let num_v_params = v_params.len();

        // 頂点パラメータをインターリーブド配列で生成: [u0,v0, u1,v1, ...]
        // グリッド順（u方向優先）で各頂点の(u,v)を格納
        let num_vertices = num_u_params * num_v_params;
        let mut vertex_params = Vec::with_capacity(num_vertices * 2);

        for &u in &u_params {
            for &v in &v_params {
                vertex_params.push(u);
                vertex_params.push(v);
            }
        }

        // u方向ノットベクトルの変換
        let u_knots: Vec<f32> = surface.u_knots().iter().map(|k| k.to_f32()).collect();

        // v方向ノットベクトルの変換
        let v_knots: Vec<f32> = surface.v_knots().iter().map(|k| k.to_f32()).collect();

        // 制御点グリッドのflatten: coordinates()で直接フラット配列を取得
        let control_points: Vec<f32> = surface.coordinates().iter().map(|c| c.to_f32()).collect();

        // 重みの変換（Foundationトレイトメソッド使用）
        let weights = surface
            .weights()
            .map(|w_flat| w_flat.iter().map(|w| w.to_f32()).collect());

        tracing::debug!(
            "📋 NurbsSurfaceEvalData 変換完了: vertices={}, control_points={}x{}, u_degree={}, v_degree={}",
            num_vertices,
            u_count,
            v_count,
            u_degree,
            v_degree
        );

        Self {
            vertex_params,
            control_points,
            weights,
            u_knots,
            v_knots,
            u_degree,
            v_degree,
            u_count,
            v_count,
            num_u_params,
            num_v_params,
        }
    }

    /// 頂点総数を取得
    pub fn num_vertices(&self) -> usize {
        self.num_u_params * self.num_v_params
    }

    /// 三角形数を取得
    pub fn num_triangles(&self) -> usize {
        let u_segs = self.num_u_params.saturating_sub(1);
        let v_segs = self.num_v_params.saturating_sub(1);
        u_segs * v_segs * 2
    }

    /// 三角形インデックスバッファ生成
    ///
    /// グリッド状のパラメータ評価結果を三角形メッシュに変換
    /// 各四角形を2つの三角形に分割（CCW巻き）
    pub fn generate_indices(&self) -> Vec<u32> {
        let u_len = self.num_u_params as u32;
        let v_len = self.num_v_params as u32;

        if u_len < 2 || v_len < 2 {
            return Vec::new();
        }

        let num_triangles = self.num_triangles();
        let mut indices = Vec::with_capacity(num_triangles * 3);

        for u_idx in 0..(u_len - 1) {
            for v_idx in 0..(v_len - 1) {
                let i0 = u_idx * v_len + v_idx;
                let i1 = i0 + 1;
                let i2 = (u_idx + 1) * v_len + v_idx;
                let i3 = i2 + 1;

                // 三角形1: CCW順（∂S/∂u × ∂S/∂v が外向き法線）
                indices.push(i0);
                indices.push(i2);
                indices.push(i1);

                // 三角形2: CCW順
                indices.push(i1);
                indices.push(i2);
                indices.push(i3);
            }
        }

        indices
    }

    /// ワイヤーフレーム用ラインインデックス生成
    ///
    /// u方向およびv方向の等パラメータ線を生成
    pub fn generate_wireframe_indices(&self) -> Vec<u32> {
        let u_len = self.num_u_params as u32;
        let v_len = self.num_v_params as u32;

        let mut indices = Vec::new();

        // u方向の線（v固定）
        for u_idx in 0..(u_len - 1) {
            for v_idx in 0..v_len {
                let i0 = u_idx * v_len + v_idx;
                let i1 = (u_idx + 1) * v_len + v_idx;
                indices.push(i0);
                indices.push(i1);
            }
        }

        // v方向の線（u固定）
        for u_idx in 0..u_len {
            for v_idx in 0..(v_len - 1) {
                let i0 = u_idx * v_len + v_idx;
                let i1 = i0 + 1;
                indices.push(i0);
                indices.push(i1);
            }
        }

        indices
    }
}
