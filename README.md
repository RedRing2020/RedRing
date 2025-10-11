# RedRing

Rust 製 CAD/CAM 研究用プラットフォーム
現在は描画基盤と構造設計の準備段階です。CAM 処理は未実装です。

**Documentation Languages / ドキュメント言語:**
| Geometry Abstraction | Link |
|----------------------|------|
| English (placeholder) | `model/GEOMETRY_README.md` |
| 日本語 (詳細) | `model/GEOMETRY_README.ja.md` |

---

## 🔍 概要

RedRing は、Rust + wgpu を用いたスタンドアロン型の CAD/CAM 開発環境を目指した研究プロジェクトです。
NURBS やプリミティブ形状などの幾何要素は未実装であり、今後段階的に導入予定です。
また、切削シミュレーションや CAM パス生成などの機能も将来的な開発対象です。

---

## 🚧 現在の開発状況

- 描画基盤（wgpu / winit）と構造設計の初期構築を進行中
- 幾何要素や CAM 処理は未実装（今後の開発対象）

- 実装進捗や設計方針は以下の Issue 一覧をご参照ください
  👉 [Issue 一覧を見る](https://github.com/RedRing2020/RedRing/issues)

- 開発中の構造や責務分離の設計は GitHub Projects で管理しています
  👉 [プロジェクトビューを見る](https://github.com/RedRing2020/RedRing/projects)

> ※ README は安定した機能が実装されたタイミングでのみ更新します。詳細な進捗は Issue/Projects をご確認ください。

---

## 📚 ドキュメントガイド

| ドキュメント                                                       | 対象読者               | 内容                                           |
| ------------------------------------------------------------------ | ---------------------- | ---------------------------------------------- |
| `README.md`                                                        | 一般利用者・新規開発者 | プロジェクト概要・ビルド方法                   |
| [`ARCHITECTURE.md`](ARCHITECTURE.md)                               | 開発者                 | ワークスペース構成・移行ステータス・テスト戦略 |
| [`manual/philosophy.md`](manual/philosophy.md)                     | コントリビューター     | 設計思想・エラー処理ガイドライン・実装パターン |
| [`model/GEOMETRY_README.ja.md`](model/GEOMETRY_README.ja.md)       | 幾何ライブラリ開発者   | 幾何抽象化の詳細仕様                           |
| [GitHub Issues](https://github.com/RedRing2020/RedRing/issues)     | 開発者                 | 機能リクエスト・バグ報告・進捗管理             |
| [GitHub Projects](https://github.com/RedRing2020/RedRing/projects) | 開発者                 | 開発ロードマップ・タスク管理                   |

---

## 🛠️ 使用技術（主要スタック）

- Rust（最新 stable 推奨）
- wgpu（GPU レンダリング）
- winit（ウィンドウ管理）
- WebAssembly（将来的に対応予定）

---

## 📋 設計方針

RedRing は**型安全性**、**責務分離**、**将来拡張性**を重視した設計を採用しています。

詳細な設計思想、エラー処理ガイドライン、トレイト設計パターンについては、以下のドキュメントをご参照ください：

📖 **[設計思想・技術指針](manual/philosophy.md)** - 開発者向け詳細ガイド

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
```

---
