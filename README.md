# RedRing

Rust製 CAD/CAM 研究用プラットフォーム  
現在は描画基盤と構造設計の準備段階です。CAM処理は未実装です。

**Documentation Languages / ドキュメント言語:**
| Geometry Abstraction | Link |
|----------------------|------|
| English (placeholder) | `model/GEOMETRY_README.md` |
| 日本語 (詳細) | `model/GEOMETRY_README.ja.md` |

---

## 🔍 概要

RedRingは、Rust + wgpu を用いたスタンドアロン型のCAD/CAM開発環境を目指した研究プロジェクトです。  
NURBSやプリミティブ形状などの幾何要素は未実装であり、今後段階的に導入予定です。  
また、切削シミュレーションやCAMパス生成などの機能も将来的な開発対象です。

---

## 🚧 現在の開発状況

- 描画基盤（wgpu / winit）と構造設計の初期構築を進行中  
- 幾何要素やCAM処理は未実装（今後の開発対象）

- 実装進捗や設計方針は以下のIssue一覧をご参照ください  
  👉 [Issue一覧を見る](https://github.com/RedRing2020/RedRing/issues)

- 開発中の構造や責務分離の設計はGitHub Projectsで管理しています  
  👉 [プロジェクトビューを見る](https://github.com/RedRing2020/RedRing/projects)

> ※ READMEは安定した機能が実装されたタイミングでのみ更新します。詳細な進捗はIssue/Projectsをご確認ください。

### 🧱 アーキテクチャ層 (現行・移行中)

```
model / analysis  ->  geo_primitives  ->  geo_core
                     (f64 canonical)     (tolerance / robust)
```

- `geo_core`: 許容誤差 (`ToleranceContext`), トレラント比較, ロバスト幾何判定 (orientation など) を提供する幾何計算中核。
- `geo_primitives`: f64 正準幾何プリミティブ（Point / Vector / Direction / LineSegment / Plane / Circle 等）。旧 Scalar ベース 3D プリミティブは移行中で内部 `Legacy*` 名に退避。
- 上位層 (`model`, `analysis`): 高次の曲線・曲面・解析アルゴリズム（今後拡張）。

### 🔄 移行ステータス (f64 Canonical Geometry)
| 項目 | 状態 | 説明 |
|------|------|------|
| Vector/Point f64 化 | 完了 | `.value()` 呼び出し不要 |
| 3D 基本プリミティブ抽出 | 進行中 | 旧実装は Legacy 命名へリネーム済み |
| f64 正準型 alias 公開 | 完了 | 旧 API 名は f64 実装へ透過接続 |
| Legacy 削除フェーズ | 未着手 | feature gate / CI deny deprecated 予定 |

詳細な移行履歴と予定は `MIGRATION_VECTOR_F64.md` の末尾「Core Role Realignment」を参照してください。

### ⚠️ 互換性ポリシー (暫定)
- 旧 `geo_core::LineSegment3D` / `Plane` / `Circle3D` / `Direction3D` を利用している場合は `geo_primitives` からの import に切替を推奨。
- 次のマイルストーンで旧名前 (legacy feature 無効ビルド) に deprecation warning / deny を導入予定。
- f64 正準層では座標アクセサは全て `f64` を返却し、距離/面積など測定量のみ `Scalar` (単位意味付け) を維持。

### 🧪 テスト戦略（要約）
- f64 ベース幾何 (ベクトル / 点 / 方向 / 線分 / 平面 / 円) に最小ユニットテストを追加済み。
- 今後: レガシー排除前に alias 経由 API の smoke test を追加予定。

---

---

## 🛠️ 使用技術（主要スタック）

- Rust（最新 stable 推奨）
- wgpu（GPUレンダリング）
- winit（ウィンドウ管理）
- WebAssembly（将来的に対応予定）

---

## 🚀 ビルド方法

### 必要環境

- Rust（最新の stable 推奨）
- cargo
- Visual Studio Build Tools（Windows の場合）

### ビルド手順

```bash
git clone https://github.com/RedRing2020/RedRing.git
cd RedRing
cargo run
