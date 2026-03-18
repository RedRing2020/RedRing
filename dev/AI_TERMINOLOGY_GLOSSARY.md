# AI Terminology Glossary (RedRing)

最終更新: 2026-03-19

このファイルは、AIの表記ゆれを防ぐための用語統一リストです。
アーキテクチャ説明・PR説明・コードコメントで共通利用します。

## 基本方針

- Rust文脈では `trait` を優先語とする
- 曖昧語より、型・責務が明確な語を使う
- 既存コードと競合する略語は新規導入しない

## 推奨語（Preferred）

- trait定義
- trait実装
- 依存関係
- 互換層
- 移設
- 段階移行
- 表記統一

## 非推奨語（Avoid）

- 契約定義
- 契約実装

## 言い換えルール

- 契約定義 -> trait定義
- 契約実装 -> trait実装
- 仕様の契約（一般論） -> 仕様（必要時のみ「契約」を補助語として使用）

## 例

- NG: `BasicCollisionの契約定義を追加`
- OK: `BasicCollisionのtrait定義を追加`

- NG: `契約実装をgeo_algorithmsへ移動`
- OK: `trait実装をgeo_algorithmsへ移動`
