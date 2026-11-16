//! geo_step - STEP（ISO 10303-21）のデータ交換
//!
//! 読み込みを段に分けて持つ（設計: BREP_AND_STEP_IMPORT_DESIGN.md 4 章）。ファイルを開かず、バイト列を受ける。
//! 入力を信用せず、上限を超えたらエラーを返し、パニックしない。
//!
//! - `syntax`: 構文（バイト列を、エンティティの表にする）
//! - `record`: 中間データ（エンティティの表を、幾何と位相の記録にする）
//! - `geometry`: 幾何の記録を、曲線・曲面の型にする
//! - `topology`: 位相の記録と型にした幾何から、B-rep の表を作る

#![cfg_attr(
    not(test),
    deny(
        clippy::unwrap_used,
        clippy::expect_used,
        clippy::panic,
        clippy::indexing_slicing
    )
)]

pub mod geometry;
pub mod record;
pub mod syntax;
pub mod topology;
