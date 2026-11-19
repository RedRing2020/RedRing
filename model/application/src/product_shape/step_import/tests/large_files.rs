//! 大きなファイルの試験（設計: BREP_AND_STEP_IMPORT_DESIGN.md 11 章）。公開されている STEP のファイルは
//! リポジトリに置かず、`tests/fixtures/step/large_files.txt` の一覧（SHA-256 とパス）と、`README.md` の取得の手順
//! だけを置く
//!
//! 取得したフォルダを環境の変数 `REDRING_STEP_LARGE_FILES` に渡して、`cargo test --release -p application --lib
//! large_files -- --nocapture` で実行する。変数がなければ、何も読まずに終わる。

use sha2::{Digest, Sha256};

use super::super::super::shell_mesh::triangulate_brep;
use super::*;

/// 三角形に分ける許容弦誤差（mm）
const CHORD_TOLERANCE: f64 = 0.01;

/// 一覧の行（SHA-256 と、フォルダからのパス）
fn listed_files() -> Vec<(String, String)> {
    let list = include_str!("../../../../../../tests/fixtures/step/large_files.txt");
    list.lines()
        .filter(|line| !line.is_empty() && !line.starts_with('#'))
        .map(|line| {
            let (hash, path) = line.split_once(' ').unwrap();
            (hash.to_string(), path.to_string())
        })
        .collect()
}

#[test]
fn public_large_files_are_read_and_triangulated() {
    let Ok(folder) = std::env::var("REDRING_STEP_LARGE_FILES") else {
        return;
    };
    for (hash, path) in listed_files() {
        let full_path = std::path::Path::new(&folder).join(&path);
        let bytes = std::fs::read(&full_path).unwrap_or_else(|error| panic!("{path}: {error}"));
        let actual: String = Sha256::digest(&bytes)
            .iter()
            .map(|byte| format!("{byte:02x}"))
            .collect();
        assert_eq!(actual, hash, "{path}: the file differs from the list");
        let (shells, report) =
            read_step(&bytes, &path).unwrap_or_else(|error| panic!("{path}: {error}"));
        let mut faces = 0;
        let mut failures: std::collections::BTreeMap<String, usize> = Default::default();
        for shell in &shells {
            let mesh = triangulate_brep(&shell.brep, CHORD_TOLERANCE);
            faces += mesh.faces.len();
            for error in mesh.faces.iter().filter_map(|face| face.as_ref().err()) {
                *failures.entry(error.to_string()).or_default() += 1;
            }
        }
        let failed_faces: usize = failures.values().sum();
        println!(
            "{path}: {report:?}, faces {faces}, faces not triangulated {failed_faces} {failures:?}"
        );
        assert!(report.shells > 0, "{path}");
        assert_eq!(failed_faces, 0, "{path}: {failures:?}");
    }
}
