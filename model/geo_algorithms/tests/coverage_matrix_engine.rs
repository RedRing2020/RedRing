use std::collections::{HashMap, HashSet};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum Dimension {
    D2,
    D3,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum Operation {
    Collision,
    Intersection,
    Distance,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum Cardinality {
    Single,
    Multiple,
    Optional,
}

#[derive(Debug, Clone)]
struct MatrixEntry {
    id: &'static str,
    dimension: Dimension,
    operation: Operation,
    shape_a: &'static str,
    shape_b: &'static str,
    required: bool,
    symmetric: bool,
    cardinality: Option<Cardinality>,
    entrypoint_a_to_b: Option<&'static str>,
    entrypoint_b_to_a: Option<&'static str>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct MatrixKey {
    dimension: Dimension,
    operation: Operation,
    shape_a: &'static str,
    shape_b: &'static str,
}

impl MatrixEntry {
    fn key(&self) -> MatrixKey {
        MatrixKey {
            dimension: self.dimension,
            operation: self.operation,
            shape_a: self.shape_a,
            shape_b: self.shape_b,
        }
    }

    fn reverse_key(&self) -> MatrixKey {
        MatrixKey {
            dimension: self.dimension,
            operation: self.operation,
            shape_a: self.shape_b,
            shape_b: self.shape_a,
        }
    }
}

fn detect_duplicate_keys(entries: &[MatrixEntry]) -> Result<(), String> {
    let mut seen = HashSet::new();
    let mut duplicates = Vec::new();

    for entry in entries {
        let key = entry.key();
        if !seen.insert(key) {
            duplicates.push(format!(
                "{:?}:{:?}:{}:{}",
                key.dimension, key.operation, key.shape_a, key.shape_b
            ));
        }
    }

    if duplicates.is_empty() {
        Ok(())
    } else {
        Err(format!(
            "duplicate matrix keys:\n- {}",
            duplicates.join("\n- ")
        ))
    }
}

fn detect_missing_required(entries: &[MatrixEntry]) -> Result<(), String> {
    let missing: Vec<&str> = entries
        .iter()
        .filter(|e| e.required && e.entrypoint_a_to_b.is_none())
        .map(|e| e.id)
        .collect();

    if missing.is_empty() {
        Ok(())
    } else {
        Err(format!(
            "missing required entrypoints:\n- {}",
            missing.join("\n- ")
        ))
    }
}

fn detect_invalid_cardinality(entries: &[MatrixEntry]) -> Result<(), String> {
    let mut invalid = Vec::new();

    for entry in entries {
        match entry.operation {
            Operation::Intersection => {
                if entry.cardinality.is_none() {
                    invalid.push(format!("{}: intersection requires cardinality", entry.id));
                }
            }
            Operation::Collision | Operation::Distance => {
                if entry.cardinality.is_some() {
                    invalid.push(format!(
                        "{}: non-intersection must have null cardinality",
                        entry.id
                    ));
                }
            }
        }
    }

    if invalid.is_empty() {
        Ok(())
    } else {
        Err(format!(
            "invalid cardinality rules:\n- {}",
            invalid.join("\n- ")
        ))
    }
}

fn detect_missing_symmetric_entrypoints(entries: &[MatrixEntry]) -> Result<(), String> {
    let missing: Vec<String> = entries
        .iter()
        .filter(|e| e.symmetric && (e.entrypoint_a_to_b.is_none() || e.entrypoint_b_to_a.is_none()))
        .map(|e| e.id.to_string())
        .collect();

    if missing.is_empty() {
        Ok(())
    } else {
        Err(format!(
            "missing symmetric entrypoints:\n- {}",
            missing.join("\n- ")
        ))
    }
}

fn detect_symmetry_mismatch(entries: &[MatrixEntry]) -> Result<(), String> {
    let by_key: HashMap<MatrixKey, &MatrixEntry> = entries.iter().map(|e| (e.key(), e)).collect();
    let mut mismatches = Vec::new();

    for entry in entries {
        if !entry.symmetric {
            continue;
        }

        match by_key.get(&entry.reverse_key()) {
            None => mismatches.push(format!(
                "missing reverse pair for {} ({} -> {})",
                entry.id, entry.shape_a, entry.shape_b
            )),
            Some(reverse) => {
                if reverse.cardinality != entry.cardinality {
                    mismatches.push(format!(
                        "cardinality mismatch {} <-> {} ({:?} vs {:?})",
                        entry.id, reverse.id, entry.cardinality, reverse.cardinality
                    ));
                }
            }
        }
    }

    if mismatches.is_empty() {
        Ok(())
    } else {
        Err(format!("symmetry mismatch:\n- {}", mismatches.join("\n- ")))
    }
}

fn validate_matrix(entries: &[MatrixEntry]) -> Result<(), String> {
    let mut errors = Vec::new();

    if let Err(err) = detect_duplicate_keys(entries) {
        errors.push(err);
    }
    if let Err(err) = detect_missing_required(entries) {
        errors.push(err);
    }
    if let Err(err) = detect_invalid_cardinality(entries) {
        errors.push(err);
    }
    if let Err(err) = detect_missing_symmetric_entrypoints(entries) {
        errors.push(err);
    }
    if let Err(err) = detect_symmetry_mismatch(entries) {
        errors.push(err);
    }

    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors.join("\n\n"))
    }
}

fn valid_seed_entries() -> Vec<MatrixEntry> {
    vec![
        MatrixEntry {
            id: "2d:circle-ray:collision",
            dimension: Dimension::D2,
            operation: Operation::Collision,
            shape_a: "Circle2D",
            shape_b: "Ray2D",
            required: true,
            symmetric: true,
            cardinality: None,
            entrypoint_a_to_b: Some("circle2d_ray2d_collides"),
            entrypoint_b_to_a: Some("ray2d_circle2d_collides"),
        },
        MatrixEntry {
            id: "2d:ray-circle:collision",
            dimension: Dimension::D2,
            operation: Operation::Collision,
            shape_a: "Ray2D",
            shape_b: "Circle2D",
            required: true,
            symmetric: true,
            cardinality: None,
            entrypoint_a_to_b: Some("ray2d_circle2d_collides"),
            entrypoint_b_to_a: Some("circle2d_ray2d_collides"),
        },
        MatrixEntry {
            id: "3d:line-segment:intersection",
            dimension: Dimension::D3,
            operation: Operation::Intersection,
            shape_a: "InfiniteLine3D",
            shape_b: "LineSegment3D",
            required: true,
            symmetric: false,
            cardinality: Some(Cardinality::Optional),
            entrypoint_a_to_b: Some("infinite_line3d_line_segment3d_intersection"),
            entrypoint_b_to_a: None,
        },
    ]
}

#[test]
fn matrix_engine_passes_for_valid_seed() {
    let entries = valid_seed_entries();
    assert!(validate_matrix(&entries).is_ok());
}

#[test]
fn matrix_engine_fails_for_missing_required_entrypoint() {
    let mut entries = valid_seed_entries();
    entries.push(MatrixEntry {
        id: "2d:segment-ray:intersection",
        dimension: Dimension::D2,
        operation: Operation::Intersection,
        shape_a: "LineSegment2D",
        shape_b: "Ray2D",
        required: true,
        symmetric: false,
        cardinality: Some(Cardinality::Single),
        entrypoint_a_to_b: None,
        entrypoint_b_to_a: None,
    });

    let err = validate_matrix(&entries).unwrap_err();
    assert!(err.contains("missing required entrypoints"));
    assert!(err.contains("2d:segment-ray:intersection"));
}

#[test]
fn matrix_engine_fails_for_symmetry_mismatch() {
    let entries = vec![
        MatrixEntry {
            id: "2d:circle-ray:intersection",
            dimension: Dimension::D2,
            operation: Operation::Intersection,
            shape_a: "Circle2D",
            shape_b: "Ray2D",
            required: true,
            symmetric: true,
            cardinality: Some(Cardinality::Multiple),
            entrypoint_a_to_b: Some("circle2d_ray2d_intersections"),
            entrypoint_b_to_a: Some("ray2d_circle2d_intersections"),
        },
        MatrixEntry {
            id: "2d:ray-circle:intersection",
            dimension: Dimension::D2,
            operation: Operation::Intersection,
            shape_a: "Ray2D",
            shape_b: "Circle2D",
            required: true,
            symmetric: true,
            cardinality: Some(Cardinality::Single),
            entrypoint_a_to_b: Some("ray2d_circle2d_intersections"),
            entrypoint_b_to_a: Some("circle2d_ray2d_intersections"),
        },
    ];

    let err = validate_matrix(&entries).unwrap_err();
    assert!(err.contains("symmetry mismatch"));
    assert!(err.contains("cardinality mismatch"));
}

#[test]
fn matrix_engine_fails_for_duplicate_registration() {
    let mut entries = valid_seed_entries();
    entries.push(MatrixEntry {
        id: "2d:circle-ray:collision:dup",
        dimension: Dimension::D2,
        operation: Operation::Collision,
        shape_a: "Circle2D",
        shape_b: "Ray2D",
        required: false,
        symmetric: false,
        cardinality: None,
        entrypoint_a_to_b: Some("circle2d_ray2d_collides"),
        entrypoint_b_to_a: None,
    });

    let err = validate_matrix(&entries).unwrap_err();
    assert!(err.contains("duplicate matrix keys"));
    assert!(err.contains("Circle2D:Ray2D"));
}

#[test]
fn matrix_engine_fails_for_missing_reverse_pair() {
    let entries = vec![MatrixEntry {
        id: "2d:segment-ray:collision",
        dimension: Dimension::D2,
        operation: Operation::Collision,
        shape_a: "LineSegment2D",
        shape_b: "Ray2D",
        required: true,
        symmetric: true,
        cardinality: None,
        entrypoint_a_to_b: Some("line_segment2d_ray2d_collides"),
        entrypoint_b_to_a: Some("ray2d_line_segment2d_collides"),
    }];

    let err = validate_matrix(&entries).unwrap_err();
    assert!(err.contains("symmetry mismatch"));
    assert!(err.contains("missing reverse pair"));
}

#[test]
fn matrix_engine_fails_when_intersection_cardinality_is_missing() {
    let entries = vec![MatrixEntry {
        id: "2d:circle-ray:intersection",
        dimension: Dimension::D2,
        operation: Operation::Intersection,
        shape_a: "Circle2D",
        shape_b: "Ray2D",
        required: true,
        symmetric: false,
        cardinality: None,
        entrypoint_a_to_b: Some("circle2d_ray2d_intersections"),
        entrypoint_b_to_a: None,
    }];

    let err = validate_matrix(&entries).unwrap_err();
    assert!(err.contains("invalid cardinality rules"));
    assert!(err.contains("intersection requires cardinality"));
}

#[test]
fn matrix_engine_fails_when_non_intersection_has_cardinality() {
    let entries = vec![MatrixEntry {
        id: "2d:circle-ray:distance",
        dimension: Dimension::D2,
        operation: Operation::Distance,
        shape_a: "Circle2D",
        shape_b: "Ray2D",
        required: true,
        symmetric: false,
        cardinality: Some(Cardinality::Optional),
        entrypoint_a_to_b: Some("circle2d_ray2d_distance"),
        entrypoint_b_to_a: None,
    }];

    let err = validate_matrix(&entries).unwrap_err();
    assert!(err.contains("invalid cardinality rules"));
    assert!(err.contains("non-intersection must have null cardinality"));
}

#[test]
fn matrix_engine_fails_when_symmetric_entrypoint_pair_is_missing() {
    let entries = vec![MatrixEntry {
        id: "2d:circle-ray:collision",
        dimension: Dimension::D2,
        operation: Operation::Collision,
        shape_a: "Circle2D",
        shape_b: "Ray2D",
        required: true,
        symmetric: true,
        cardinality: None,
        entrypoint_a_to_b: Some("circle2d_ray2d_collides"),
        entrypoint_b_to_a: None,
    }];

    let err = validate_matrix(&entries).unwrap_err();
    assert!(err.contains("missing symmetric entrypoints"));
    assert!(err.contains("2d:circle-ray:collision"));
}
