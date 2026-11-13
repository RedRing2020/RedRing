# 依存クレートの検査の運用

最終更新: 2026-11-05

## 目的

依存クレートの既知の脆弱性と、ライセンス・取得元の決まりの違反を、CI で見つけて直す。

## 検査の中身

`cargo deny`（設定は `deny.toml`）で、次の 4 つを検査する。

| 検査 | 内容 |
|---|---|
| advisories | RustSec の勧告（既知の脆弱性・保守の終了・健全性の注意・取り下げた版） |
| licenses | 依存のライセンスが、許可の一覧（本体のライセンス MIT OR Apache-2.0 と両立するもの）に入ること |
| bans | ワイルドカードの版の指定をしないこと。同じクレートの複数の版は警告だけ |
| sources | 取得元が crates.io であること（ほかのレジストリ・Git を使わない） |

- RustSec の勧告は `cargo audit` と同じデータベースを使うため、`cargo audit` は別に回さない
- 本体のクレートは crates.io に公開しないため、すべて `publish = false` とする（ワークスペースの中のパスの依存を、ワイルドカードの検査の対象から外すため）。本体のクレートのライセンスは、ワークスペースのライセンス（`license.workspace = true`）とする

## いつ回すか

| きっかけ | 内容 |
|---|---|
| 依存を変える PR・push（`Cargo.lock`・`Cargo.toml`・`deny.toml`） | 4 つの検査。advisories は、PR では失敗にしない（依存を変えていない勧告で PR を止めないため） |
| 週に 1 回（月曜 9:00） | 4 つの検査。依存を変えていなくても、新しく見つかった脆弱性に気づける |
| 手元 | 依存を変えるときは、push の前に `cargo deny check` を回す（`cargo install cargo-deny --locked` で入れる） |

ワークフローは `.github/workflows/dependency_audit.yml`。

## 見つかった場合の扱い

1. 脆弱性・健全性の注意・取り下げた版: 直る版へ上げる。互換のある版で直る場合は `cargo update -p <クレート>`（`Cargo.lock` だけの変更）とする。互換のない版が要る場合は、`Cargo.toml` の版を上げ、試験を通す
2. 直る版がない・上流の対応を待つ場合: 影響（実際に使う所か、ビルドのときだけか、信用できる入力だけか）を確かめ、`deny.toml` の `[advisories] ignore` に、勧告の ID と理由を書いて残す。見直しのため Issue を起票する
3. 保守の終了: 置き換えの候補を確かめ、直接の依存なら置き換えを Issue にする。間接の依存で上流の対応を待つ場合は、2. と同じく理由を書いて残す
4. 新しいライセンス: 本体のライセンスと両立するかを確かめてから、`deny.toml` の許可の一覧に足す。両立しないものは、その依存を使わない
5. 新しい取得元: crates.io 以外は原則として使わない。使う場合は、理由とともに `deny.toml` の許可に足す

## Dependabot

`.github/dependabot.yml` で、Cargo の依存と GitHub Actions の更新を、週に 1 回 `develop` へ PR で知らせる。

- 互換のある更新（マイナー・パッチ）は 1 つの PR にまとめ、PR とレビューの数を抑える
- Dependabot の PR も、ほかの PR と同じく CI（ビルド・試験・依存の検査）が通ってからマージする
- 互換のない更新（0.x のマイナー・メジャー）は、変更の内容を確かめ、必要なら Issue にして対応する

## Rust の版

Rust の版は、リポジトリの直下の `rust-toolchain.toml` の 1 か所で決める（#1279）。rustup はリポジトリの中ではこのファイルに従うため、手元・CI・リリースのビルドが同じ版になる。stable が上がっても、CI の結果は変わらない（新しい Clippy の指摘で、急に CI が落ちることがない）。

- CI・リリースのワークフローは、版を書かずに、共通の手順 `.github/actions/setup-rust`（引数なしの `rustup toolchain install`）で `rust-toolchain.toml` の版と部品を入れる。cargo-deny は自分のコンテナで動くため、`rust-toolchain.toml` の版を読んで Action に渡す
- Cargo.toml には `rust-version` を書かない（版を書く場所を 2 つにしないため）

版を上げるときは、次の順に行う。

1. `rust-toolchain.toml` の `channel` を新しい版にする
2. `pwsh -NoProfile -File scripts/check_all_before_push.ps1` を回し、新しい Clippy の指摘・警告を直す
3. 版を上げる変更と、指摘を直す変更を 1 つの PR にする（CHANGELOG に版を書く）

## GitHub Actions の決まり

- 第三者の Action は、コミットの SHA で固定し、版を行末のコメントに書く（例: `uses: actions/checkout@<SHA> # v4.4.0`）。タグやブランチ（`@v4`・`@master`）は付け替えられるため使わない
- ツールの版（mdBook 等）も、`latest` ではなく版を決めて書く
- 版の更新は、Dependabot の知らせ（SHA とコメントの版をあわせて上げる）で行う。メジャーの版を上げる場合は、変更の内容を確かめる
- ワークフローの既定の権限は `contents: read` だけとし、書き込みの権限（`pages: write`・`id-token: write` 等）は、それが要るジョブだけに与える
- 保守の終わった Action は使わない

