# Transform システム

RedRingの幾何変換システムについて説明します。Transform trait 定義とエラー型は `geo_core` に集約され、各図形クレートはそれらを再公開して利用します。

## 設計思想

### 設計原則

- **Analysis統合**: `analysis`クレートのMatrix4x4/Vector3を直接使用
- **型変換効率**: geo_primitives⇔analysis間の最適化された変換
- **2層責務分離**: 共通Transform責務は `geo_core`、形状適用は各図形クレート
- **エラーハンドリング**: TransformErrorによる安全な変換操作
- **汎用座標変換**: 支点や平行移動量は `analysis::Vector2/Vector3` で明示指定

### シンプルな構成

```text
現在の構成:
├── AnalysisTransform3D        - 3D座標点変換
├── AnalysisTransformVector3D  - 3D方向ベクトル変換
├── AnalysisTransform2D        - 2D変換
└── TransformError             - エラーハンドリング

基盤:
├── analysis::Matrix4x4        - 変換行列計算
├── analysis::Vector3          - 3Dベクトル操作
└── analysis::Vector2          - 2Dベクトル操作
```

## Core Traits

### AnalysisTransform3D

3D座標点の変換を担当するメイントレイト：

```rust
pub trait AnalysisTransform3D<T: Scalar> {
    type Matrix4x4;  // analysis::Matrix4x4
    type Angle;      // analysis::Angle
    type Output;     // 通常はSelf

    // 直接Matrix変換
    fn transform_point_matrix(&self, matrix: &Self::Matrix4x4) -> Self::Output;

    // 基本変換操作
    fn translate_analysis(&self, translation: &Vector3<T>) -> Result<Self::Output, TransformError>;
    fn rotate_analysis(&self, center: &Vector3<T>, axis: &Vector3<T>, angle: Self::Angle) -> Result<Self::Output, TransformError>;
    fn scale_analysis(&self, center: &Vector3<T>, scale_x: T, scale_y: T, scale_z: T) -> Result<Self::Output, TransformError>;
    fn uniform_scale_analysis(&self, center: &Vector3<T>, scale_factor: T) -> Result<Self::Output, TransformError>;

    // 複合変換
    fn apply_composite_transform(
        &self,
        translation: Option<&Vector3<T>>,
        rotation: Option<(&Vector3<T>, &Vector3<T>, Self::Angle)>,
        scale: Option<(T, T, T)>
    ) -> Result<Self::Output, TransformError>;
}
```

### AnalysisTransformVector3D

方向ベクトル専用変換（平行移動成分を自動的に無視）：

```rust
pub trait AnalysisTransformVector3D<T: Scalar> {
    // 方向ベクトル変換（平行移動無視）
    fn transform_vector_matrix(&self, matrix: &Self::Matrix4x4) -> Self::Output;
    fn rotate_vector_analysis(&self, axis: &Vector3<T>, angle: Self::Angle) -> Result<Self::Output, TransformError>;
    fn scale_vector_analysis(&self, scale_x: T, scale_y: T, scale_z: T) -> Result<Self::Output, TransformError>;

    // Analysis正規化
    fn normalize_analysis(&self) -> Result<Self::Output, TransformError>;
}
```

## 実装例

### TriangleMesh3D Transform

効率的なメッシュ変換の実装例：

```rust
impl<T: Scalar> AnalysisTransform3D<T> for TriangleMesh3D<T> {
    type Matrix4x4 = Matrix4x4<T>;
    type Angle = Angle<T>;
    type Output = Self;

    fn transform_point_matrix(&self, matrix: &Matrix4x4<T>) -> Self {
        let mut transformed_vertices = Vec::with_capacity(self.vertices().len());

        for vertex in self.vertices() {
            // 効率的な型変換チェーン
            let vertex_vec = vertex.to_analysis_vector3();          // Point3D → Vector3
            let transformed_vec = matrix.transform_point_3d(&vertex_vec);
            let new_vertex = Point3D::from_analysis_vector3(transformed_vec);  // Vector3 → Point3D
            transformed_vertices.push(new_vertex);
        }

        TriangleMesh3D::new(transformed_vertices, self.indices().to_vec())
            .unwrap_or_else(|_| TriangleMesh3D::empty())
    }
}
```

### 効率的な型変換

geo_primitives型とanalysis型間の最適化された変換：

```rust
impl<T: Scalar> Point3D<T> {
    // analysis統合変換
    pub fn to_analysis_vector3(&self) -> analysis::Vector3<T> {
        self.to_analysis_point3().to_vector()
    }

    pub fn from_analysis_vector3(v: analysis::Vector3<T>) -> Self {
        let point = analysis::Point3::from_vector(v);
        Self::from_analysis_point3(point)
    }

    // 直接変換
    pub fn to_analysis_point3(&self) -> analysis::Point3<T> {
        analysis::Point3::new(self.x, self.y, self.z)
    }
}
```

## 使用例

詳細なコード例は [transform_examples.md](./transform_examples.md) を参照してください。

## エラーハンドリング

統一されたTransformErrorによる堅牢なエラー処理：

```rust
use geo_core::TransformError;

match mesh.rotate_analysis(&center, &axis, angle) {
    Ok(rotated) => { /* 成功 */ }
    Err(TransformError::ZeroVector(msg)) => {
        eprintln!("Invalid rotation axis: {}", msg);
    }
    Err(TransformError::InvalidRotation(msg)) => {
        eprintln!("Rotation error: {}", msg);
    }
    Err(e) => {
        eprintln!("Other transform error: {:?}", e);
    }
}
```

## パフォーマンス特徴

- **ゼロコピー最適化**: 中間オブジェクト生成を最小化
- **Analysis統合**: 高度に最適化されたMatrix/Vector演算を直接活用
- **型変換効率**: geo_primitives⇔analysis間の最適化された変換チェーン
- **メモリ効率**: 不要な中間配列やコピーを排除

## 対応図形

現在実装済みの図形：

- ✅ **TriangleMesh3D** - メッシュ変換
- ✅ **Direction3D** - 方向ベクトル変換
- ✅ **CylindricalSolid3D** - 円柱ソリッド変換
- ✅ **CylindricalSurface3D** - 円柱サーフェス変換
- ✅ **ConicalSurface3D** - 円錐サーフェス変換
- ✅ **EllipsoidalSurface3D** - 楕円体サーフェス変換

追加実装は各図形の `*_transform.rs` ファイルで段階的に展開予定。

## 閉発状況

### 現在の実装状況

✅ **完成済み**:
- `AnalysisTransform3D` - 3D座標点変換トレイト
- `AnalysisTransformVector3D` - 3D方向ベクトル変換トレイト
- `TransformError` - 統一エラーハンドリング
- Analysis統合 - Matrix4x4/Vector3直接使用

⚙️ **開発中**:
- `AnalysisTransform2D` - 2D変換トレイト
- 追加の幾何プリミティブ対応

### 設計の特徴

```rust
// 現在のシンプルな設計
trait AnalysisTransform3D<T: Scalar> {
    type Matrix4x4;  // analysis::Matrix4x4<T>
    type Angle;      // analysis::Angle<T>
    type Output;     // 通常はSelf

    // 直接Matrix変換
    fn transform_point_matrix(&self, matrix: &Self::Matrix4x4) -> Self::Output;

    // Analysisベースの字全な変換操作
    fn translate_analysis(&self, translation: &Vector3<T>) -> Result<Self::Output, TransformError>;
    fn rotate_analysis(&self, center: &Vector3<T>, axis: &Vector3<T>, angle: Self::Angle) -> Result<Self::Output, TransformError>;
    fn scale_analysis(&self, center: &Vector3<T>, scale_x: T, scale_y: T, scale_z: T) -> Result<Self::Output, TransformError>;
}
```

この設計により、analysisクレートの高性能なMatrix/Vector演算を直接活用し、同時に存全なエラーハンドリングを提供しています。
