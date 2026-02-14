//! NURBS GPU評価用のデータ変換
//!
//! Model層（geo_nurbs）のNURBS形状とパラメータリストを、
//! View層（render）でGPU評価可能な形式に変換します。

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
        curve: &geo_nurbs::NurbsCurve3D<T>,
        param_list: &geo_nurbs::adaptive_tessellation::AdaptiveParamList<T>,
    ) -> Self {
        use geo_foundation::NurbsCurve3DProperties;

        let degree = curve.degree() as u32;
        
        // ノットベクトルの変換
        let knots: Vec<f32> = curve
            .knot_vector()
            .iter()
            .map(|k| {
                // Scalar::to_f32()はfallibleだが、通常のf64->f32変換なら問題ない
                // 念のため、既にf32の場合と分ける
                k.to_f32()
            })
            .collect();

        // 制御点のflatten: control_point(i)を全て取得してflatten
        let num_cp = curve.control_points_count();
        let mut control_points = Vec::with_capacity(num_cp * 3);
        for i in 0..num_cp {
            let cp = curve.control_point(i);
            control_points.push(cp.x().to_f32());
            control_points.push(cp.y().to_f32());
            control_points.push(cp.z().to_f32());
        }

        // 重みの変換（NurbsCurve3DPropertiesトレイトメソッド使用）  
        let weights = <geo_nurbs::NurbsCurve3D<T> as NurbsCurve3DProperties<T>>::weights(curve)
            .map(|w_slice| w_slice.iter().map(|wi| wi.to_f32()).collect());

        // パラメータの変換（Vec<T> -> Vec<f32>）
        let params: Vec<f32> = param_list
            .params
            .iter()
            .map(|p| p.to_f32())
            .collect();

        tracing::info!(
            "📋 NurbsCurveEvalData 変換完了: {} params, {} control points, degree={}, {} knots",
            params.len(),
            num_cp,
            degree,
            knots.len()
        );
        
        if !control_points.is_empty() {
            tracing::info!(
                "📋 制御点[0]: ({:.3}, {:.3}, {:.3})",
                control_points[0],
                control_points[1],
                control_points[2]
            );
            if num_cp > 1 {
                let last_idx = control_points.len() - 3;
                tracing::info!(
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
    use geo_nurbs::adaptive_tessellation::{
        AdaptiveTessellationSettings, NurbsCurveAdaptiveTessellation,
    };
    use geo_nurbs::NurbsCurve3D;

    #[test]
    fn test_nurbs_curve_eval_data_from_line() {
        // 直線のNURBS曲線を作成（(0,0,0) -> (1,0,0)）
        let curve = NurbsCurve3D::<f64>::line_segment((0.0, 0.0, 0.0), (1.0, 0.0, 0.0)).unwrap();

        // 適応パラメータ生成
        let settings = AdaptiveTessellationSettings::default_with_tolerance(0.01);
        let param_list = curve.adaptive_params_curve(&settings);

        // GPU評価用データに変換
        let eval_data = NurbsCurveEvalData::from_curve_params(&curve, &param_list);

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
