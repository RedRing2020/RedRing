# ARCHITECTURE.md から移した記録（完了した移行計画・解消した課題）

**移動日**: 2026年10月12日
**移動元**: `dev/architecture/ARCHITECTURE.md`

`ARCHITECTURE.md` を現在の構成と規則の正本として整理する際に、完了した移行計画と、その後の整理で解消・置き換えられた課題・方針をここへ移した。現在の構成と規則は `ARCHITECTURE.md` を参照する。

## 移した時点での状況

- `geo_foundation` の廃止と、形状の trait定義の `geo_contracts` への移設は完了している
- 形状の trait定義は `geo_contracts` に置き、形状のクレートは再エクスポートしない（「形状API統一の実現」の各形状クレート内の trait 経由という目標とは異なる）
- trait定義のタプルの受け渡しの廃止は、値型の新クレート化とあわせて別途扱う
- 点・ベクトルは `T: Scalar` のジェネリック型で、f64 に固定していない
- 参照していた `MIGRATION_VECTOR_F64.md` / `model/GEOMETRY_README.ja.md` / `GITHUB_PAGES_SETUP.md` は現存しない

---

## 🚨 現在の重大な課題（2026年3月15日更新）

### 系統的な*_core_traits実装問題

10以上の形状で**レガシーAPIとFoundation実装が共存**し、以下の問題が発生しています：

- **メソッド名競合**: `center()`, `arc_length()`, `point_at_parameter()` 等
- **型不一致**: `Point2D<T>` vs `(T, T)` のシグネチャ違い
- **Foundation Patternの破綻**: 統一アクセスが実現できない

**対象形状**: point, vector, circle, ellipse_arc, direction, bbox, ray, line_segment, infinite_line

### アーキテクチャ再編提案（geo_foundation 廃止）

```text
foundation/analysis（将来改名候補）
            ↓
        geo_contracts（trait定義の正本）
          ↓        ↓
 geo_commons     geo_core（低レイヤー・基本型提供）
  （analysisのみ依存の   ↓             ↓
   数値カーネル）  geo_primitives      geo_nurbs
                      （impl入口）       （impl入口）
                            ↘           ↙
                    geo_algorithms（交差/衝突/幾何演算）
                    ↓
         application層クレート群
    （tessellation / simulation / job manager）

        CAM側の責務分離（設計更新）

            geo_*（汎用幾何演算）
                ↓
            cam_core（中立データ基盤）
               ↙          ↘
        cam_algorithms      cam_entity
        （CAM固有計算）     （表示/統合）
               ↓
              cam_sim
        （CAM特化ユースケース実行）
```

### 段階移行計画（提案）

1. **Phase 1: trait移設**
- `geo_foundation` の形状traitを `geo_contracts` へ移設
- 既存利用側を新trait参照へ置換

2. **Phase 2: アルゴリズム統合**
- `geo_algorithms` に `geo_primitives` と `geo_nurbs` の両依存を明示
- Intersect/Collision等のクロス形状演算を `geo_algorithms` に統一

3. **Phase 3: application層分離**
- tessellation/simulation/job manager を application層クレートに移動
- `geo_algorithms` は純粋幾何アルゴリズム責務に限定
- 初期導入は PoC から段階適用とし、初手は単一受け皿 + module 分割を優先する
- `*_orchestration` の独立クレート化は、責務境界と依存分岐が安定した段階で判断する

4. **Phase 4: analysis再編**
- `foundation/analysis` の名称変更を検討
- 内部実装を純粋数値解析（線形代数等）に整理し、ドメイン責務を排除

## 🔧 修正方針

### レガシーAPIの段階的置き換え

1. **競合メソッドの内部化**: レガシー `pub fn` を `fn` に変更
2. **Foundation実装の真の移行**: 共存ではなく置き換えアプローチ
3. **最小限の実装から開始**: 1-2個のメソッドから段階的に実装
4. **geo_commons機能の活用**: 共通計算で内部実装を再利用

### 形状API統一の実現（新方針）

**目標**: 全てのアクセスを「各形状クレート内trait」経由に統一

```rust
// ✅ 目標: 形状クレート内trait経由のみアクセス可能
use geo_primitives::EllipseArc2DDerived;

let arc = EllipseArc2D::new(...);
let length = arc.length(); // trait実装を呼び出し
let point = arc.point_at_parameter(0.5); // trait実装を呼び出し

// ❌ 禁止: レガシー直接アクセス
// arc.legacy_method() // コンパイルエラー
```

#### アプリケーション層の責務

- **`render`**: GPU 描画基盤（wgpu + WGSL）- 基本描画機能のみ
- **`stage`**: レンダリングステージ管理 - 最小限の構造のみ
- **`viewmodel`**: ビュー操作・変換ロジック - 基礎機能のみ
- **`redring`**: メインアプリケーション - ウィンドウ表示のみ

**現在未実装の主要機能**:

- メニューシステム
- コマンドパレット
- ファイル操作（開く/保存）
- WebAssembly対応
- 実用的なCAD/CAM機能

## 🔄 f64正準化移行について

- **基本方針**: Vector/Point は f64 正準型、測定量は Scalar `<T>` 維持
- **Legacy型**: 全て削除済み、CI で deprecated symbols を deny
- **詳細履歴**: `MIGRATION_VECTOR_F64.md` を参照

## 📝 重要な教訓

1. **共存アプローチの失敗**: レガシーとFoundationの共存はメソッド名競合を引き起こす
2. **置き換えの必要性**: Foundation Patternの真の価値は統一アクセスにある
3. **段階的実装の重要性**: 一度に多数のメソッドを実装すると失敗する

## 🎆 期待される成果

- **統一アクセス**: 全てのAPIが各形状クレートのtrait経由
- **型安全性**: コンパイル時のインターフェース統一
- **保守性向上**: 明確な責務分離と依存関係
- **拡張性**: 新しい形状追加は各形状クレート、クロス演算は`geo_algorithms`に集約

