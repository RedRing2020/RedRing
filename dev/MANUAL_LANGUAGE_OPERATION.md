# 利用者向けマニュアルの日本語・英語の運用

最終更新: 2026-11-07

## 構成

| 言語 | 原稿 | 設定 | 出力（GitHub Pages） |
|---|---|---|---|
| 日本語 | `manual/` | ルートの `book.toml` | `target/book/`（`https://redring2020.github.io/RedRing/`） |
| 英語 | `manual_en/src/` | `manual_en/book.toml` | `target/book_en/`。公開では `target/book/en/` に置く（`https://redring2020.github.io/RedRing/en/`） |

- 2 冊は別々にビルドする（`mdbook build`・`mdbook build manual_en`）。公開する CI（`main_ci_cd.yml`）は、英語の出力を日本語の出力の `en` に写してから公開する。マニュアル・`book.toml` を変える PR では、`manual_ci.yml` が両方がビルドできることを確かめる（文書だけの変更では、`develop_ci.yml` のビルド・試験は回らない）
- 両方の「はじめに」の先頭に、もう一方の言語のマニュアルへのリンクを置く（日本語から `en/index.html`、英語から `../index.html`。公開した形で通じる）
- アプリの「ヘルプ」→「バージョンの情報」のマニュアルへのリンクは、画面の言語のマニュアルを開く（URL は文言表の `about.manual.url`）
- mdBook の版は CI と同じ（0.5.4）を使う。0.5 で使えない設定の項目があると、`[output.html]` の全体が読まれないため、設定を変えたら出力（テーマ・編集のリンク）を確かめる

## そろえる決まり

- 日本語と英語は、同じページ・同じ見出しの構成にし、内容をそろえる。片方を直す PR では、もう片方も同じ PR で直す
- 画面の文言（メニュー・ボタン・ウィンドウの名前）は、画面の文言表の各言語の文言と同じ言葉を使う（英語は `view/app/src/*_messages.rs` の英語の表）
- 開発者向けの設計書（`dev/`）・CHANGELOG は日本語だけとする。英語のマニュアルから参照するときは「in Japanese」と添える

## リリースノート

- GitHub のリリースの本文は、日本語の後に英語を置く（`dev/RELEASE_NOTE_TEMPLATE.md`）
- `dev/releases/` の記録は日本語だけでよい
