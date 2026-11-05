# はじめに

RedRing は、Rust + wgpu による CAD/CAM 研究用プラットフォームです。
型安全性と責務分離を重視した設計で、幾何計算の基盤から CAM のツールパス生成・切削シミュレーションまでを開発しています。

## 背景と目的

従来の CAD カーネルは、複雑な依存関係と不明瞭な責務分担により、保守性と拡張性に課題がありました。
RedRing は、Rust の型システムを活用し、**型安全性・責務分離・将来拡張性**を重視したモダンなアーキテクチャを目指しています。

## 設計方針

- ✅ **型安全性**：コンパイル時エラー検出による堅牢な設計
- 🧩 **責務分離**：各モジュールが単一の責務を持つクリーンアーキテクチャ（画面・ViewModel・Application・Model の層に分ける）
- 🌍 **国際化対応**：画面の文言は日本語・英語に対応
- ⚡ **パフォーマンス**：wgpu による GPU 加速レンダリング
- ✅ **拡張可能性**：トレイトによる抽象化で将来機能の追加に対応

## 現在の構成

RedRing は、次の層のワークスペースになっています。

### 基盤（foundation）

- `analysis`：数値解析・許容誤差
- `logging_foundation`・`i18n_foundation`：ログの初期化・文言の表

### 幾何（model の geo）

- `geo_contracts`：幾何の trait 定義の正本
- `geo_core`・`geo_commons`：幾何の値型・形状に共通の数値計算
- `geo_primitives`・`geo_nurbs`：解析形状・NURBS 曲線と曲面
- `geo_topology`：位相構造
- `geo_algorithms`：形状をまたぐ演算（距離・交差・テセレーション）
- `geo_entity`：エンティティの基盤（識別子・属性・関係）
- `geo_io`：ファイルの入出力（STL 等）

### CAM（model の cam）

- `cam_core`：CAM の中立データ（ツールパス・工具・成果物の形式）
- `cam_algorithms`：ツールパス生成（逆オフセット法のスキャン加工）
- `cam_entity`：加工手順・加工工程のエンティティ
- `cam_sim`：切削シミュレーション（トリデクセル）

### ジョブとユースケース

- `job_runtime`・`job_domain`：ジョブの実行制御・投入の決まり
- `application`：ユースケースの入口（ViewModel から Model への唯一の入口）

### 画面

- `viewmodel`・`viewmodel-graphics`：表示用の変換・カメラ等の制御
- `render`・`stage`：GPU 描画基盤・レンダリングステージ
- `redring`：メインアプリケーション

設計の詳細は、リポジトリの [アーキテクチャ構成](https://github.com/RedRing2020/RedRing/blob/develop/dev/architecture/ARCHITECTURE.md) と、[設計書の一覧](https://github.com/RedRing2020/RedRing/tree/develop/dev/architecture) を参照してください。

## 対象読者

- RedRing のプレビュー版を試す方（[動かす](./getting_started.md) から始めてください）
- CAD/CAM プラットフォーム開発に関心のある Rust 開発者
- 幾何計算・GPU レンダリングの技術者
- オープンソース CAD プロジェクトへのコントリビューター

## 開発状況

**🔒 v0.1.1 プレビュー**（2026年11月）: v0.1.0 の修正版です。セキュリティの修正（細工した成果物・STL のファイルでアプリが落ちる問題の修正、依存クレートの既知の脆弱性の修正）を含みます。v0.1.0 をお使いの場合は、v0.1.1 をお使いください。加工の条件は、加工手順・加工工程として持つようになり、取り込みは、今の条件と表示中の製品形状で計算した結果だけになりました（[切削シミュレーション 操作](./cutting_simulation_demo.md)）。

**🔪 v0.1.0 プレビュー**（2026年11月）: STL を取り込み、3 軸のスキャン加工のツールパスを計算し、切削シミュレーションで確かめられます。NC の出力はまだないため、実際の加工には使えません。

次の版（v0.2.0）では、加工の管理（加工工程の一覧・ジョブマネージャー・工具の登録）を予定しています。

最新の進捗状況は以下で確認できます：

- [GitHub Issues](https://github.com/RedRing2020/RedRing/issues)
- [変更の記録（CHANGELOG）](https://github.com/RedRing2020/RedRing/blob/develop/CHANGELOG.md)

---

このマニュアルは、プレビュー版を使う方に向けたものです。開発者向けの説明（設計・クレートの構成・API）は、リポジトリの `dev/architecture/` を正本とし、マニュアルには置きません。
