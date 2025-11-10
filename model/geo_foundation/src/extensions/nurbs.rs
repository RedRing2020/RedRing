//! NURBS特有の拡張操作
//!
//! NURBS曲線・サーフェスに特化した高度な操作を定義

use crate::Scalar;

/// NURBS曲線の高度な操作
pub trait NurbsCurveExtensions<T: Scalar> {
    /// エラー型
    type Error;

    /// 結果の曲線型
    type CurveResult;

    /// ノット挿入
    ///
    /// # 引数
    /// * `parameter` - 挿入するパラメータ値
    /// * `multiplicity` - 重複度（通常1）
    ///
    /// # 戻り値
    /// ノットが挿入された新しい曲線
    fn insert_knot(
        &self,
        parameter: T,
        multiplicity: usize,
    ) -> Result<Self::CurveResult, Self::Error>;

    /// 次数昇格
    ///
    /// # 引数
    /// * `target_degree` - 目標次数
    ///
    /// # 戻り値
    /// 次数が昇格された新しい曲線
    fn elevate_degree(&self, target_degree: usize) -> Result<Self::CurveResult, Self::Error>;

    /// 曲線分割
    ///
    /// # 引数
    /// * `parameter` - 分割点のパラメータ値
    ///
    /// # 戻り値
    /// 分割された2つの曲線のタプル
    fn split_at(&self, parameter: T)
        -> Result<(Self::CurveResult, Self::CurveResult), Self::Error>;

    /// 制御点挿入
    ///
    /// # 引数
    /// * `index` - 挿入位置
    /// * `point` - 挿入する制御点
    /// * `weight` - 重み（有理曲線の場合）
    ///
    /// # 戻り値
    /// 制御点が挿入された新しい曲線
    fn insert_control_point(
        &self,
        index: usize,
        point: Self::Point,
        weight: Option<T>,
    ) -> Result<Self::CurveResult, Self::Error>;

    /// 制御点削除
    ///
    /// # 引数
    /// * `index` - 削除する制御点のインデックス
    ///
    /// # 戻り値
    /// 制御点が削除された新しい曲線
    fn remove_control_point(&self, index: usize) -> Result<Self::CurveResult, Self::Error>;

    /// 曲率計算
    ///
    /// # 引数
    /// * `parameter` - 曲率を計算するパラメータ値
    ///
    /// # 戻り値
    /// 指定点での曲率
    fn curvature_at(&self, parameter: T) -> T;

    /// 曲線反転
    ///
    /// # 戻り値
    /// パラメータ方向が反転された新しい曲線
    fn reverse(&self) -> Self::CurveResult;

    /// 曲線の均等分割
    ///
    /// # 引数
    /// * `num_segments` - 分割数
    ///
    /// # 戻り値
    /// 均等分割されたパラメータ値のベクトル
    fn uniform_parameters(&self, num_segments: usize) -> Vec<T>;

    /// 弧長パラメータ化
    ///
    /// # 引数
    /// * `arc_length` - 弧長
    ///
    /// # 戻り値
    /// 指定弧長に対応するパラメータ値
    fn parameter_from_arc_length(&self, arc_length: T) -> Option<T>;

    /// Point型の定義
    type Point;
}

/// NURBSサーフェスの高度な操作
pub trait NurbsSurfaceExtensions<T: Scalar> {
    /// エラー型
    type Error;

    /// 結果のサーフェス型
    type SurfaceResult;

    /// 結果の曲線型（等パラメータ線抽出用）
    type CurveResult;

    /// u方向ノット挿入
    ///
    /// # 引数
    /// * `parameter` - 挿入するu方向パラメータ値
    /// * `multiplicity` - 重複度
    ///
    /// # 戻り値
    /// ノットが挿入された新しいサーフェス
    fn insert_u_knot(
        &self,
        parameter: T,
        multiplicity: usize,
    ) -> Result<Self::SurfaceResult, Self::Error>;

    /// v方向ノット挿入
    ///
    /// # 引数
    /// * `parameter` - 挿入するv方向パラメータ値
    /// * `multiplicity` - 重複度
    ///
    /// # 戻り値
    /// ノットが挿入された新しいサーフェス
    fn insert_v_knot(
        &self,
        parameter: T,
        multiplicity: usize,
    ) -> Result<Self::SurfaceResult, Self::Error>;

    /// u方向次数昇格
    ///
    /// # 引数
    /// * `target_degree` - 目標次数
    ///
    /// # 戻り値
    /// u方向次数が昇格された新しいサーフェス
    fn elevate_u_degree(&self, target_degree: usize) -> Result<Self::SurfaceResult, Self::Error>;

    /// v方向次数昇格
    ///
    /// # 引数
    /// * `target_degree` - 目標次数
    ///
    /// # 戻り値
    /// v方向次数が昇格された新しいサーフェス
    fn elevate_v_degree(&self, target_degree: usize) -> Result<Self::SurfaceResult, Self::Error>;

    /// u方向分割
    ///
    /// # 引数
    /// * `parameter` - 分割点のu方向パラメータ値
    ///
    /// # 戻り値
    /// u方向で分割された2つのサーフェスのタプル
    fn split_u_at(
        &self,
        parameter: T,
    ) -> Result<(Self::SurfaceResult, Self::SurfaceResult), Self::Error>;

    /// v方向分割
    ///
    /// # 引数
    /// * `parameter` - 分割点のv方向パラメータ値
    ///
    /// # 戻り値
    /// v方向で分割された2つのサーフェスのタプル
    fn split_v_at(
        &self,
        parameter: T,
    ) -> Result<(Self::SurfaceResult, Self::SurfaceResult), Self::Error>;

    /// u方向等パラメータ線抽出
    ///
    /// # 引数
    /// * `u_parameter` - 固定するu方向パラメータ値
    ///
    /// # 戻り値
    /// 抽出された曲線
    fn extract_u_curve(&self, u_parameter: T) -> Result<Self::CurveResult, Self::Error>;

    /// v方向等パラメータ線抽出
    ///
    /// # 引数
    /// * `v_parameter` - 固定するv方向パラメータ値
    ///
    /// # 戻り値
    /// 抽出された曲線
    fn extract_v_curve(&self, v_parameter: T) -> Result<Self::CurveResult, Self::Error>;

    /// 境界線抽出
    ///
    /// # 戻り値
    /// 4つの境界曲線（u_min, u_max, v_min, v_max）
    fn boundary_curves(&self) -> Result<[Self::CurveResult; 4], Self::Error>;

    /// ガウス曲率計算
    ///
    /// # 引数
    /// * `u` - u方向パラメータ値
    /// * `v` - v方向パラメータ値
    ///
    /// # 戻り値
    /// 指定点でのガウス曲率
    fn gaussian_curvature_at(&self, u: T, v: T) -> T;

    /// 平均曲率計算
    ///
    /// # 引数
    /// * `u` - u方向パラメータ値
    /// * `v` - v方向パラメータ値
    ///
    /// # 戻り値
    /// 指定点での平均曲率
    fn mean_curvature_at(&self, u: T, v: T) -> T;

    /// 主曲率計算
    ///
    /// # 引数
    /// * `u` - u方向パラメータ値
    /// * `v` - v方向パラメータ値
    ///
    /// # 戻り値
    /// 指定点での主曲率（最大・最小）
    fn principal_curvatures_at(&self, u: T, v: T) -> (T, T);

    /// サーフェス反転
    ///
    /// # 引数
    /// * `reverse_u` - u方向を反転するか
    /// * `reverse_v` - v方向を反転するか
    ///
    /// # 戻り値
    /// パラメータ方向が反転された新しいサーフェス
    fn reverse(&self, reverse_u: bool, reverse_v: bool) -> Self::SurfaceResult;

    /// サーフェスの均等分割パラメータ
    ///
    /// # 引数
    /// * `u_segments` - u方向分割数
    /// * `v_segments` - v方向分割数
    ///
    /// # 戻り値
    /// 均等分割されたパラメータ値のグリッド
    fn uniform_parameters(&self, u_segments: usize, v_segments: usize) -> Vec<Vec<(T, T)>>;

    /// Point型とVector型の定義
    type Point;
    type Vector;
}

/// NURBS重み操作の拡張
pub trait NurbsWeightExtensions<T: Scalar> {
    /// エラー型
    type Error;

    /// 結果のジオメトリ型
    type GeometryResult;

    /// 重みを正規化
    ///
    /// # 戻り値
    /// 重みが正規化された新しいジオメトリ
    fn normalize_weights(&self) -> Self::GeometryResult;

    /// 特定重みの変更
    ///
    /// # 引数
    /// * `index` - 変更する制御点のインデックス
    /// * `new_weight` - 新しい重み値
    ///
    /// # 戻り値
    /// 重みが変更された新しいジオメトリ
    fn set_weight(&self, index: usize, new_weight: T) -> Result<Self::GeometryResult, Self::Error>;

    /// 重み配列を一括設定
    ///
    /// # 引数
    /// * `weights` - 新しい重み配列
    ///
    /// # 戻り値
    /// 重みが設定された新しいジオメトリ
    fn set_weights(&self, weights: Vec<T>) -> Result<Self::GeometryResult, Self::Error>;

    /// 非有理化（全重みを1.0に）
    ///
    /// # 戻り値
    /// 非有理化された新しいジオメトリ
    fn make_non_rational(&self) -> Self::GeometryResult;

    /// 重みの統計情報
    ///
    /// # 戻り値
    /// (最小重み, 最大重み, 平均重み)
    fn weight_statistics(&self) -> (T, T, T);
}

/// NURBS幾何のテッセレーション・メッシュ化
pub trait NurbsTessellation<T: Scalar> {
    /// 結果の点型
    type Point;

    /// 結果の三角形型
    type Triangle;

    /// 結果のメッシュ型
    type Mesh;

    /// 均等分割による点群生成
    ///
    /// # 引数
    /// * `resolution` - 分解能（曲線の場合は点数、サーフェスの場合は各方向の点数）
    ///
    /// # 戻り値
    /// 生成された点群
    fn tessellate_points(&self, resolution: usize) -> Vec<Self::Point>;

    /// 三角形メッシュ生成（サーフェス用）
    ///
    /// # 引数
    /// * `u_resolution` - u方向分解能
    /// * `v_resolution` - v方向分解能
    ///
    /// # 戻り値
    /// 生成された三角形メッシュ
    fn tessellate_triangles(&self, u_resolution: usize, v_resolution: usize)
        -> Vec<Self::Triangle>;

    /// 高品質メッシュ生成
    ///
    /// # 引数
    /// * `tolerance` - 許容誤差
    /// * `max_subdivisions` - 最大細分割数
    ///
    /// # 戻り値
    /// 適応的に生成された高品質メッシュ
    fn adaptive_mesh(&self, tolerance: T, max_subdivisions: usize) -> Self::Mesh;
}
