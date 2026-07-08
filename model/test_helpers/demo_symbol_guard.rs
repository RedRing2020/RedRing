use std::fs;
use std::path::{Path, PathBuf};

const FORBIDDEN_DEMO_SYMBOLS: [&str; 5] = [
    "CamSimulationDemoScenario",
    "build_demo_artifacts_for_cam_simulation",
    "create_demo_snapshot_exports_for_scenario",
    "create_sample_snapshot_exports_for_demo",
    "cam_sim_demo",
];

fn collect_rs_files(dir: &Path, files: &mut Vec<PathBuf>) {
    assert!(
        dir.exists(),
        "source directory does not exist: {}",
        dir.display()
    );

    for entry in fs::read_dir(dir).unwrap_or_else(|error| {
        panic!("source directory should be readable: {} ({})", dir.display(), error)
    }) {
        let entry = entry.unwrap_or_else(|error| {
            panic!("directory entry should be readable in {}: {}", dir.display(), error)
        });
        let path = entry.path();
        if path.is_dir() {
            collect_rs_files(&path, files);
        } else if path.extension().is_some_and(|ext| ext == "rs") {
            files.push(path);
        }
    }
}

pub fn assert_layer_does_not_reference_demo_symbols(layer_name: &str) {
    let src_root = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    let mut files = Vec::new();
    collect_rs_files(&src_root, &mut files);
    files.sort();

    for file in files {
        let content = fs::read_to_string(&file).unwrap_or_else(|error| {
            panic!("source file should be readable: {} ({})", file.display(), error)
        });
        for symbol in FORBIDDEN_DEMO_SYMBOLS {
            assert!(
                !content.contains(symbol),
                "{} layer must not reference demo symbol '{}' in {}",
                layer_name,
                symbol,
                file.display()
            );
        }
    }
}