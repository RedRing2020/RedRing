# RedRing

**🇺🇸 [English README is here](README.md)**

Rust 製 CAD/CAM 研究用プラットフォーム

現在は描画基盤と構造設計の準備段階です。CAM 処理は未実装です。

**Documentation Languages / ドキュメント言語:**

| ドキュメント               | 言語                     | アクセス方法                                                  |
| -------------------------- | ------------------------ | ------------------------------------------------------------- |
| **オンラインドキュメント** | 🌐 日本語                | **[📖 GitHub Pages](https://redring2020.github.io/RedRing/)** |
| Geometry Abstraction       | 🇺🇸 English (placeholder) | `model/GEOMETRY_README.md`                                    |
| Geometry Abstraction       | 🇯🇵 日本語 (詳細)         | `model/GEOMETRY_README.ja.md`                                 |

---

## 🔍 概要

RedRing は、Rust + wgpu を用いたスタンドアロン型の CAD/CAM 開発環境を目指した研究プロジェクトです。
NURBS やプリミティブ形状などの幾何要素は現在開発中であり、段階的に導入予定です。
また、切削シミュレーションや CAM パス生成などの機能も将来的な開発対象です。

### 🌟 特徴

- **Rust による型安全性**: メモリ安全性とパフォーマンスを両立
- **GPU レンダリング**: wgpu による高性能な3D描画基盤
- **モジュラー設計**: 責務分離された拡張可能なアーキテクチャ
- **国際化対応**: 多言語でのドキュメンテーション
- **研究志向**: オープンソースによる CAD/CAM 技術の探求

---

## 🚧 現在の開発状況

### ✅ 実装済み機能

- 🎨 **描画基盤** (wgpu/winit) - GPU レンダリングパイプライン
- 📐 **基礎幾何ライブラリ** - 点、線、面、ベクトル演算
- 🏗️ **Foundation パターン** - 統一された幾何プリミティブインターフェース
- 📊 **NURBS システム** - NURBS 曲線・曲面の完全実装
- 🔧 **テスト基盤** - 包括的なテストスイート (23/23 テスト通過)
- 📖 **ドキュメンテーション** - mdbook による美しい技術文書

### 🔄 開発中機能

- 🎯 幾何アルゴリズムの拡張
- 🎮 インタラクティブなユーザーインターフェース
- 💾 CAD ファイル形式のサポート (STEP/IGES)

### 📅 今後の予定

- 🔪 CAM パス生成エンジン
- ⚡ 切削シミュレーション
- 🌐 WebAssembly サポート
- 🖱️ SpaceMouse 対応

実装進捗や設計方針の詳細は以下をご参照ください：

- 👉 [Issue 一覧を見る](https://github.com/RedRing2020/RedRing/issues)
- 👉 [プロジェクトビューを見る](https://github.com/RedRing2020/RedRing/projects)

> **注意:** README は安定した機能が実装されたタイミングでのみ更新します。詳細な進捗は Issue/Projects をご確認ください。

---

## 📚 ドキュメントガイド

| ドキュメント                                                       | 対象読者                 | 内容                                           |
| ------------------------------------------------------------------ | ------------------------ | ---------------------------------------------- |
| `README.jp.md` (本文書)                                            | 日本語利用者・新規開発者 | プロジェクト概要・ビルド方法                   |
| `README.md`                                                        | 英語圏利用者・国際開発者 | Project Overview & Build Instructions          |
| [`ARCHITECTURE.md`](ARCHITECTURE.md)                               | 開発者                   | ワークスペース構成・移行ステータス・テスト戦略 |
| [`manual/philosophy.md`](manual/philosophy.md)                     | コントリビューター       | 設計思想・エラー処理ガイドライン・実装パターン |
| [`model/GEOMETRY_README.ja.md`](model/GEOMETRY_README.ja.md)       | 幾何ライブラリ開発者     | 幾何抽象化の詳細仕様                           |
| [`.github/AI_DEV_GUIDE.md`](.github/AI_DEV_GUIDE.md)               | 🤖 AI開発者              | セッション復旧・開発継続支援                   |
| [GitHub Issues](https://github.com/RedRing2020/RedRing/issues)     | 開発者                   | 機能リクエスト・バグ報告・進捗管理             |
| [GitHub Projects](https://github.com/RedRing2020/RedRing/projects) | 開発者                   | 開発ロードマップ・タスク管理                   |

---

## 🛠️ 使用技術（主要スタック）

### コア技術

- **Rust** (最新 stable 推奨) - システムプログラミング言語
- **wgpu** - クロスプラットフォーム GPU API
- **winit** - ウィンドウ管理・イベント処理

### 数値計算・幾何処理

- **nalgebra** - 線形代数ライブラリ
- **approx** - 浮動小数点比較
- **カスタム NURBS** - 自社実装の NURBS エンジン

### 開発・テスト環境

- **cargo** - Rust パッケージマネージャ
- **mdbook** - ドキュメント生成
- **GitHub Actions** - CI/CD パイプライン

### 将来対応予定

- **WebAssembly** - ブラウザ実行環境
- **STEP/IGES** - CAD ファイル形式
- **OpenCASCADE** - 高度な幾何カーネル (検討中)

---

## 📋 設計方針

RedRing は以下の原則に基づいて設計されています：

### 🔒 型安全性

- Rust の所有権システムによるメモリ安全性
- ジェネリクス・トレイトによる抽象化
- コンパイル時エラー検出による品質向上

### 🏗️ 責務分離

- **Foundation**: 基礎機能・数値解析
- **Model**: 幾何データ層・アルゴリズム
- **View**: アプリケーション・描画層
- **ViewModel**: ビュー変換ロジック

### 🚀 将来拡張性

- モジュラーなクレート構成
- プラグイン可能なアーキテクチャ
- 段階的機能追加に対応

詳細な設計思想、エラー処理ガイドライン、トレイト設計パターンについては：

📖 **[設計思想・技術指針](manual/philosophy.md)** - 開発者向け詳細ガイド

---

## 🚀 ビルド方法

### 必要環境

#### 基本要件

- **Rust** (最新 stable 推奨) - [公式サイト](https://www.rust-lang.org/)からインストール
- **cargo** (Rust に同梱)
- **git** - リポジトリのクローン用

#### プラットフォーム固有要件

**Windows:**

- Visual Studio Build Tools または Visual Studio Community
- Windows 10/11 (DirectX 12 対応)

**macOS:**

- Xcode Command Line Tools: `xcode-select --install`
- macOS 10.15+ (Metal 対応)

**Linux:**

- 必要なパッケージ: `sudo apt install build-essential pkg-config libx11-dev`
- Vulkan または OpenGL ドライバ

### ビルド手順

#### 1. リポジトリのクローン

```bash
git clone https://github.com/RedRing2020/RedRing.git
cd RedRing
```

#### 2. 依存関係の確認

```bash
# Rust バージョン確認
rustc --version

# ビルドツールの確認
cargo --version
```

#### 3. ビルド実行

```bash
# デバッグビルド (高速)
cargo build

# リリースビルド (最適化)
cargo build --release
```

#### 4. アプリケーション実行

```bash
# GUI 環境が必要 (X11/Wayland/Windows/macOS)
cargo run

# テスト実行
cargo test --workspace
```

#### 5. ドキュメント生成 (オプション)

```bash
# mdbook が必要: cargo install mdbook
mdbook build  # manual/ -> docs/ に生成
mdbook serve  # ローカルサーバーで確認
```

### トラブルシューティング

#### GPU ドライバ関連エラー

```bash
# wgpu が GPU を検出できない場合
export WGPU_BACKEND=vulkan  # Linux
# または
export WGPU_BACKEND=dx12    # Windows
```

#### ビルドエラー

```bash
# 依存関係の更新
cargo update

# クリーンビルド
cargo clean && cargo build
```

---

## 🤝 コントリビューション

RedRing プロジェクトへの貢献を歓迎します！

### 貢献方法

1. **Issue の確認**: [GitHub Issues](https://github.com/RedRing2020/RedRing/issues) で既存の課題を確認
2. **フォーク**: リポジトリをフォークして作業用ブランチを作成
3. **実装**: 機能追加・バグ修正を実装
4. **テスト**: `cargo test --workspace` でテスト通過を確認
5. **プルリクエスト**: 変更内容を説明してプルリクエストを作成

### 開発ガイドライン

- **コードスタイル**: `cargo fmt` で自動フォーマット
- **リント**: `cargo clippy` で品質チェック
- **ドキュメント**: 公開 API には必ず rustdoc コメントを記載
- **テスト**: 新機能には対応するテストを追加

### コミュニティ

- 🐛 **バグ報告**: [Issues](https://github.com/RedRing2020/RedRing/issues) で報告
- 💡 **機能提案**: [Discussions](https://github.com/RedRing2020/RedRing/discussions) で議論
- 📖 **ドキュメント改善**: プルリクエストで提案

---

## 📜 ライセンス

このプロジェクトは [MIT License](LICENSE) の下で公開されています。

---

## 🙏 謝辞

RedRing の開発にご協力いただいているすべての貢献者に感謝いたします。

また、以下のオープンソースプロジェクトの恩恵を受けています：

- [Rust Programming Language](https://www.rust-lang.org/)
- [wgpu](https://wgpu.rs/) - WebGPU implementation
- [winit](https://github.com/rust-windowing/winit) - Window handling
- [nalgebra](https://nalgebra.org/) - Linear algebra library

---

## 🔗 関連リンク

- 📧 **連絡先**: [Issues](https://github.com/RedRing2020/RedRing/issues) または [Discussions](https://github.com/RedRing2020/RedRing/discussions)
- 🌐 **ウェブサイト**: [GitHub Pages](https://redring2020.github.io/RedRing/)
- 🐙 **GitHub**: [RedRing2020/RedRing](https://github.com/RedRing2020/RedRing)
