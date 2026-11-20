# RedRing

**🇺🇸 [English README is here](README.md)**

![CodeRabbit Pull Request Reviews](https://img.shields.io/coderabbit/prs/github/RedRing2020/RedRing?utm_source=oss&utm_medium=github&utm_campaign=RedRing2020%2FRedRing&labelColor=171717&color=FF570A&link=https%3A%2F%2Fcoderabbit.ai&label=CodeRabbit+Reviews)

RedRing は、Rust・wgpu・egui で作るスタンドアロンの CAD/CAM のアプリです。研究のプロジェクトとして開発しています。

## 今の状態

最新のリリースの v0.1.3 はプレビュー版です。STL を取り込み、ボール・フラットエンドミルで 3 軸のスキャン加工のツールパスを計算し、切削シミュレーションで確かめられます。NC の出力はまだないため、実際の加工には使えません。

これからの版の予定:

- v0.2.0（開発中）: 加工の管理（工程・ジョブ・ツールライブラリ）と、STEP の形状を加工の対象にすること
- v0.3.0: 時間のかかる計算を裏で進め、進み具合を出し、止められるようにすること
- v0.4.0: つなぎの移動と NC の出力
- v0.5.0: 面を選んで加工の範囲と座標を決めること

変更は [CHANGELOG.md](CHANGELOG.md) を、今の制約は [既知の制約](https://redring2020.github.io/RedRing/known_issues.html) を見てください。

## 起動方法

Windows では、[GitHub のリリース](https://github.com/RedRing2020/RedRing/releases) から配布物を取ってきて、`redring.exe` を起動します。

ソースからビルドするときは、[rustup](https://rustup.rs/) と Git を入れてから、次を実行します。

```bash
git clone https://github.com/RedRing2020/RedRing.git
cd RedRing
git checkout v0.1.3
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
