# RedRing

**🌐[English README](README.md)**

[![Version](https://img.shields.io/github/v/release/RedRing2020/RedRing?include_prereleases&label=release&color=blue)](https://github.com/RedRing2020/RedRing/releases)
[![Rust](https://img.shields.io/badge/dynamic/toml?url=https%3A%2F%2Fraw.githubusercontent.com%2FRedRing2020%2FRedRing%2Fdevelop%2Frust-toolchain.toml&query=%24.toolchain.channel&label=rust&logo=rust&color=orange)](rust-toolchain.toml)
[![Last Commit](https://img.shields.io/github/last-commit/RedRing2020/RedRing/develop)](https://github.com/RedRing2020/RedRing/commits/develop)
[![Commit Activity](https://img.shields.io/github/commit-activity/m/RedRing2020/RedRing)](https://github.com/RedRing2020/RedRing/graphs/commit-activity)
![License](https://img.shields.io/badge/License-MIT%2FApache--2.0-green)

RedRing は、Rust・wgpu・egui で作るスタンドアロンの CAD/CAM のアプリです。研究のプロジェクトとして開発しています。

## 今の状態

最新のリリースの v0.2.0 はプレビュー版です。STL・STEP の形状を取り込み、ツールライブラリの工具で加工工程を組み、ボール・フラットエンドミルで 3 軸のスキャン加工のツールパスをジョブマネージャーで計算し、前の工程の後の素材から削る切削シミュレーションで確かめられます。モデルファイルに保存できます。NC の出力はまだないため、実際の加工には使えません。

これからの版の予定:

- v0.3.0（開発中）: 時間のかかる計算を裏で進め、進み具合を出し、止められるようにすること
- v0.4.0: つなぎの移動と NC の出力
- v0.5.0: 面を選んで加工の範囲と座標を決めること

変更は [CHANGELOG.md](CHANGELOG.md) を、今の制約は [既知の制約](https://redring2020.github.io/RedRing/known_issues.html) を見てください。

## 起動方法

Windows では、[GitHub のリリース](https://github.com/RedRing2020/RedRing/releases) から配布物を取ってきて、`redring.exe` を起動します。

ソースからビルドするときは、[rustup](https://rustup.rs/) と Git を入れてから、次を実行します。

```bash
git clone https://github.com/RedRing2020/RedRing.git
cd RedRing
git checkout v0.2.0
cargo build --release
cargo run --release -p redring
```

`cargo build --release` で、ツールパスと切削シミュレーションを計算するジョブマネージャーとランナーも一緒にビルドします。動く環境、配布物の確かめ方、ジョブマネージャーは [動かす](https://redring2020.github.io/RedRing/getting_started.html) を見てください。

## 文書

- 最新のリリースの [利用者向けのマニュアル](https://redring2020.github.io/RedRing/)（[英語](https://redring2020.github.io/RedRing/en/)）: 動かし方、[画面と操作](https://redring2020.github.io/RedRing/screen_and_controls.html)、[切削シミュレーション](https://redring2020.github.io/RedRing/cutting_simulation_demo.html)。開発中の版のマニュアルは [manual/](manual/) にあります
- [貢献の手引き](CONTRIBUTING.jp.md): 開発の準備、push の前の検査、プルリクエスト
- [設計書](dev/architecture/): [全体の構成](dev/architecture/ARCHITECTURE.md) と部分ごとの設計
- [Issues](https://github.com/RedRing2020/RedRing/issues): 不具合の報告、機能の提案、進み具合

## ライセンス

RedRing は、[MIT License](LICENSE-MIT) と [Apache License 2.0](LICENSE-APACHE) のどちらかを選べるデュアルライセンスです。[LICENSE](LICENSE) を見てください。
