# RedRing

Rust製 CAD/CAM 研究用プラットフォーム  
現在は描画基盤と構造設計の準備段階です。CAM処理は未実装です。

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
