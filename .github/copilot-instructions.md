# Copilot Instructions for RedRing

## プロジェクト概要
- RedRingはRust製のCAD/CAM研究用プラットフォームです。
- 主な技術: Rust, wgpu (GPUレンダリング), winit (ウィンドウ管理)
- 構造設計と描画基盤が中心。幾何要素やCAM処理は今後実装予定。

## アーキテクチャと主要ディレクトリ
- `model/` : 幾何・構造データの定義（例: geometry2d, Ellipse.rs）
- `render/` : GPU描画・シェーダ関連
- `redring/` : アプリケーション本体（エントリーポイントやUI）
- `viewmodel/` : 表示・操作モデル
- `stage/` : シーン管理や構造設計
- `manual/` : ドキュメント（Markdown形式）
- `docs/` , `book/` : 静的ドキュメント/サイト生成

## ビルド・実行
- Rust最新版（stable）推奨
- Windowsの場合はVisual Studio Build Toolsが必要
- 標準的なビルド/実行コマンド:
  ```powershell
  git clone https://github.com/RedRing2020/RedRing.git
  cd RedRing
  cargo run
  ```
- cargoコマンドで各crate（model, render, redring等）を個別にビルド可能

## 開発・設計方針
- 責務分離を重視（モデル/レンダラ/ビュー/ステージ）
- Rustのモジュール/クレート分割を活用
- 主要な型・構造体は`model/src/`や`render/src/`に定義
- 依存関係はCargo.tomlで管理
- wgpu/winitのAPI設計に沿った構造

## テスト・デバッグ
- cargo testでユニットテスト実行（テストは未整備の場合あり）
- デバッグはcargo run + ログ出力（log crate推奨）

## コーディング規約・パターン
- Rust標準の命名規則（snake_case, UpperCamelCase）
- モジュール分割はsrc/配下で階層化
- 主要な幾何/描画要素は専用ファイル・モジュールで管理
- 例: `model/src/geometry/geometry2d/Ellipse.rs` に楕円の定義

## 参考・設計例
- `README.md`に全体像・技術スタック・ビルド手順あり
- 詳細な進捗や設計はGitHub Issue/Projects参照

---
このファイルはAIコーディングエージェント向けのガイドです。設計方針やディレクトリ構成、主要な開発コマンドを明記し、RedRingプロジェクトで即戦力となるための知識をまとめています。
