//! geo_step - STEP（ISO 10303-21）のデータ交換
//!
//! 読み込みを段に分けて持つ（設計: BREP_AND_STEP_IMPORT_DESIGN.md 4 章）。ファイルを開かず、バイト列を受ける。
//! 入力を信用せず、上限を超えたらエラーを返し、パニックしない。
//!
//! - `syntax`: 構文（バイト列を、エンティティの表にする）

#![cfg_attr(
    not(test),
    deny(
        clippy::unwrap_used,
        clippy::expect_used,
        clippy::panic,
        clippy::indexing_slicing
    )
)]

pub mod syntax;
