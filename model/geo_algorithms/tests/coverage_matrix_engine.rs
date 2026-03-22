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

fn collect_pub_fn_names(source: &str) -> HashSet<String> {
    source
        .lines()
        .filter_map(|line| {
            let line = line.trim_start();
            if !line.starts_with("pub fn ") {
                return None;
            }

            let rest = &line["pub fn ".len()..];
            let name: String = rest
                .chars()
                .take_while(|c| c.is_ascii_alphanumeric() || *c == '_')
                .collect();

            if name.is_empty() {
                None
            } else {
                Some(name)
            }
        })
        .collect()
}

fn known_entrypoints() -> HashSet<String> {
    let sources = [
        include_str!("../src/collision/primitive_2d.rs"),
        include_str!("../src/collision/primitive_3d.rs"),
        include_str!("../src/collision/primitive_nurbs.rs"),
        include_str!("../src/intersection/primitive_2d.rs"),
        include_str!("../src/intersection/primitive_3d.rs"),
        include_str!("../src/distance/primitive_2d.rs"),
        include_str!("../src/distance/primitive_3d.rs"),
    ];

    let mut set = HashSet::new();
    for source in sources {
        set.extend(collect_pub_fn_names(source));
    }

    set
}

fn detect_unknown_entrypoints(entries: &[MatrixEntry]) -> Result<(), String> {
    let known = known_entrypoints();
    let mut unknown = Vec::new();

    for entry in entries {
        if let Some(ep) = entry.entrypoint_a_to_b {
            if !known.contains(ep) {
                unknown.push(format!("{}: {}", entry.id, ep));
            }
        }
        if let Some(ep) = entry.entrypoint_b_to_a {
            if !known.contains(ep) {
                unknown.push(format!("{}: {}", entry.id, ep));
            }
        }
    }

    if unknown.is_empty() {
        Ok(())
    } else {
        Err(format!("unknown entrypoints:\n- {}", unknown.join("\n- ")))
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
    if let Err(err) = detect_unknown_entrypoints(entries) {
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
        },
        MatrixEntry {
            id: "2d:ray-segment:collision",
            dimension: Dimension::D2,
            operation: Operation::Collision,
            shape_a: "Ray2D",
            shape_b: "LineSegment2D",
            required: true,
            symmetric: true,
            cardinality: None,
            entrypoint_a_to_b: Some("ray2d_line_segment2d_collides"),
            entrypoint_b_to_a: Some("line_segment2d_ray2d_collides"),
        },
        MatrixEntry {
            id: "2d:arc-circle:collision",
            dimension: Dimension::D2,
            operation: Operation::Collision,
            shape_a: "Arc2D",
            shape_b: "Circle2D",
            required: true,
            symmetric: true,
            cardinality: None,
            entrypoint_a_to_b: Some("arc2d_circle2d_collides"),
            entrypoint_b_to_a: Some("circle2d_arc2d_collides"),
        },
        MatrixEntry {
            id: "2d:circle-arc:collision",
            dimension: Dimension::D2,
            operation: Operation::Collision,
            shape_a: "Circle2D",
            shape_b: "Arc2D",
            required: true,
            symmetric: true,
            cardinality: None,
            entrypoint_a_to_b: Some("circle2d_arc2d_collides"),
            entrypoint_b_to_a: Some("arc2d_circle2d_collides"),
        },
        MatrixEntry {
            id: "2d:segment-arc:collision",
            dimension: Dimension::D2,
            operation: Operation::Collision,
            shape_a: "LineSegment2D",
            shape_b: "Arc2D",
            required: true,
            symmetric: true,
            cardinality: None,
            entrypoint_a_to_b: Some("line_segment2d_arc2d_collides"),
            entrypoint_b_to_a: Some("arc2d_line_segment2d_collides"),
        },
        MatrixEntry {
            id: "2d:arc-segment:collision",
            dimension: Dimension::D2,
            operation: Operation::Collision,
            shape_a: "Arc2D",
            shape_b: "LineSegment2D",
            required: true,
            symmetric: true,
            cardinality: None,
            entrypoint_a_to_b: Some("arc2d_line_segment2d_collides"),
            entrypoint_b_to_a: Some("line_segment2d_arc2d_collides"),
        },
        MatrixEntry {
            id: "2d:triangle-circle:collision",
            dimension: Dimension::D2,
            operation: Operation::Collision,
            shape_a: "Triangle2D",
            shape_b: "Circle2D",
            required: true,
            symmetric: true,
            cardinality: None,
            entrypoint_a_to_b: Some("triangle2d_circle2d_collides"),
            entrypoint_b_to_a: Some("circle2d_triangle2d_collides"),
        },
        MatrixEntry {
            id: "2d:circle-triangle:collision",
            dimension: Dimension::D2,
            operation: Operation::Collision,
            shape_a: "Circle2D",
            shape_b: "Triangle2D",
            required: true,
            symmetric: true,
            cardinality: None,
            entrypoint_a_to_b: Some("circle2d_triangle2d_collides"),
            entrypoint_b_to_a: Some("triangle2d_circle2d_collides"),
        },
        MatrixEntry {
            id: "2d:triangle-segment:collision",
            dimension: Dimension::D2,
            operation: Operation::Collision,
            shape_a: "Triangle2D",
            shape_b: "LineSegment2D",
            required: true,
            symmetric: true,
            cardinality: None,
            entrypoint_a_to_b: Some("triangle2d_line_segment2d_collides"),
            entrypoint_b_to_a: Some("line_segment2d_triangle2d_collides"),
        },
        MatrixEntry {
            id: "2d:segment-triangle:collision",
            dimension: Dimension::D2,
            operation: Operation::Collision,
            shape_a: "LineSegment2D",
            shape_b: "Triangle2D",
            required: true,
            symmetric: true,
            cardinality: None,
            entrypoint_a_to_b: Some("line_segment2d_triangle2d_collides"),
            entrypoint_b_to_a: Some("triangle2d_line_segment2d_collides"),
        },
        MatrixEntry {
            id: "2d:line-circle:collision",
            dimension: Dimension::D2,
            operation: Operation::Collision,
            shape_a: "InfiniteLine2D",
            shape_b: "Circle2D",
            required: true,
            symmetric: true,
            cardinality: None,
            entrypoint_a_to_b: Some("infinite_line2d_circle2d_collides"),
            entrypoint_b_to_a: Some("circle2d_infinite_line2d_collides"),
        },
        MatrixEntry {
            id: "2d:circle-line:collision",
            dimension: Dimension::D2,
            operation: Operation::Collision,
            shape_a: "Circle2D",
            shape_b: "InfiniteLine2D",
            required: true,
            symmetric: true,
            cardinality: None,
            entrypoint_a_to_b: Some("circle2d_infinite_line2d_collides"),
            entrypoint_b_to_a: Some("infinite_line2d_circle2d_collides"),
        },
        MatrixEntry {
            id: "2d:line-segment:collision",
            dimension: Dimension::D2,
            operation: Operation::Collision,
            shape_a: "InfiniteLine2D",
            shape_b: "LineSegment2D",
            required: true,
            symmetric: true,
            cardinality: None,
            entrypoint_a_to_b: Some("infinite_line2d_line_segment2d_collides"),
            entrypoint_b_to_a: Some("line_segment2d_infinite_line2d_collides"),
        },
        MatrixEntry {
            id: "2d:segment-line:collision",
            dimension: Dimension::D2,
            operation: Operation::Collision,
            shape_a: "LineSegment2D",
            shape_b: "InfiniteLine2D",
            required: true,
            symmetric: true,
            cardinality: None,
            entrypoint_a_to_b: Some("line_segment2d_infinite_line2d_collides"),
            entrypoint_b_to_a: Some("infinite_line2d_line_segment2d_collides"),
        },
        MatrixEntry {
            id: "2d:line-ray:collision",
            dimension: Dimension::D2,
            operation: Operation::Collision,
            shape_a: "InfiniteLine2D",
            shape_b: "Ray2D",
            required: true,
            symmetric: true,
            cardinality: None,
            entrypoint_a_to_b: Some("infinite_line2d_ray2d_collides"),
            entrypoint_b_to_a: Some("ray2d_infinite_line2d_collides"),
        },
        MatrixEntry {
            id: "2d:ray-line:collision",
            dimension: Dimension::D2,
            operation: Operation::Collision,
            shape_a: "Ray2D",
            shape_b: "InfiniteLine2D",
            required: true,
            symmetric: true,
            cardinality: None,
            entrypoint_a_to_b: Some("ray2d_infinite_line2d_collides"),
            entrypoint_b_to_a: Some("infinite_line2d_ray2d_collides"),
        },
        MatrixEntry {
            id: "2d:ellipse-circle:collision",
            dimension: Dimension::D2,
            operation: Operation::Collision,
            shape_a: "Ellipse2D",
            shape_b: "Circle2D",
            required: true,
            symmetric: true,
            cardinality: None,
            entrypoint_a_to_b: Some("ellipse2d_circle2d_collides"),
            entrypoint_b_to_a: Some("circle2d_ellipse2d_collides"),
        },
        MatrixEntry {
            id: "2d:circle-ellipse:collision",
            dimension: Dimension::D2,
            operation: Operation::Collision,
            shape_a: "Circle2D",
            shape_b: "Ellipse2D",
            required: true,
            symmetric: true,
            cardinality: None,
            entrypoint_a_to_b: Some("circle2d_ellipse2d_collides"),
            entrypoint_b_to_a: Some("ellipse2d_circle2d_collides"),
        },
        MatrixEntry {
            id: "2d:ellipse_arc-circle:collision",
            dimension: Dimension::D2,
            operation: Operation::Collision,
            shape_a: "EllipseArc2D",
            shape_b: "Circle2D",
            required: true,
            symmetric: true,
            cardinality: None,
            entrypoint_a_to_b: Some("ellipse_arc2d_circle2d_collides"),
            entrypoint_b_to_a: Some("circle2d_ellipse_arc2d_collides"),
        },
        MatrixEntry {
            id: "2d:circle-ellipse_arc:collision",
            dimension: Dimension::D2,
            operation: Operation::Collision,
            shape_a: "Circle2D",
            shape_b: "EllipseArc2D",
            required: true,
            symmetric: true,
            cardinality: None,
            entrypoint_a_to_b: Some("circle2d_ellipse_arc2d_collides"),
            entrypoint_b_to_a: Some("ellipse_arc2d_circle2d_collides"),
        },
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
            cardinality: Some(Cardinality::Multiple),
            entrypoint_a_to_b: Some("ray2d_circle2d_intersections"),
            entrypoint_b_to_a: Some("circle2d_ray2d_intersections"),
        },
        MatrixEntry {
            id: "2d:segment-ray:intersection",
            dimension: Dimension::D2,
            operation: Operation::Intersection,
            shape_a: "LineSegment2D",
            shape_b: "Ray2D",
            required: true,
            symmetric: true,
            cardinality: Some(Cardinality::Single),
            entrypoint_a_to_b: Some("line_segment2d_ray2d_intersection"),
            entrypoint_b_to_a: Some("ray2d_line_segment2d_intersection"),
        },
        MatrixEntry {
            id: "2d:ray-segment:intersection",
            dimension: Dimension::D2,
            operation: Operation::Intersection,
            shape_a: "Ray2D",
            shape_b: "LineSegment2D",
            required: true,
            symmetric: true,
            cardinality: Some(Cardinality::Single),
            entrypoint_a_to_b: Some("ray2d_line_segment2d_intersection"),
            entrypoint_b_to_a: Some("line_segment2d_ray2d_intersection"),
        },
        MatrixEntry {
            id: "2d:circle-ellipse:intersection",
            dimension: Dimension::D2,
            operation: Operation::Intersection,
            shape_a: "Circle2D",
            shape_b: "Ellipse2D",
            required: true,
            symmetric: true,
            cardinality: Some(Cardinality::Optional),
            entrypoint_a_to_b: Some("circle2d_ellipse2d_intersection"),
            entrypoint_b_to_a: Some("ellipse2d_circle2d_intersection"),
        },
        MatrixEntry {
            id: "2d:ellipse-circle:intersection",
            dimension: Dimension::D2,
            operation: Operation::Intersection,
            shape_a: "Ellipse2D",
            shape_b: "Circle2D",
            required: true,
            symmetric: true,
            cardinality: Some(Cardinality::Optional),
            entrypoint_a_to_b: Some("ellipse2d_circle2d_intersection"),
            entrypoint_b_to_a: Some("circle2d_ellipse2d_intersection"),
        },
        MatrixEntry {
            id: "2d:line-circle:intersection",
            dimension: Dimension::D2,
            operation: Operation::Intersection,
            shape_a: "InfiniteLine2D",
            shape_b: "Circle2D",
            required: true,
            symmetric: true,
            cardinality: Some(Cardinality::Multiple),
            entrypoint_a_to_b: Some("infinite_line2d_circle2d_intersections"),
            entrypoint_b_to_a: Some("circle2d_infinite_line2d_intersections"),
        },
        MatrixEntry {
            id: "2d:circle-line:intersection",
            dimension: Dimension::D2,
            operation: Operation::Intersection,
            shape_a: "Circle2D",
            shape_b: "InfiniteLine2D",
            required: true,
            symmetric: true,
            cardinality: Some(Cardinality::Multiple),
            entrypoint_a_to_b: Some("circle2d_infinite_line2d_intersections"),
            entrypoint_b_to_a: Some("infinite_line2d_circle2d_intersections"),
        },
        MatrixEntry {
            id: "2d:line-segment:intersection",
            dimension: Dimension::D2,
            operation: Operation::Intersection,
            shape_a: "InfiniteLine2D",
            shape_b: "LineSegment2D",
            required: true,
            symmetric: true,
            cardinality: Some(Cardinality::Optional),
            entrypoint_a_to_b: Some("infinite_line2d_line_segment2d_intersection"),
            entrypoint_b_to_a: Some("line_segment2d_infinite_line2d_intersection"),
        },
        MatrixEntry {
            id: "2d:segment-line:intersection",
            dimension: Dimension::D2,
            operation: Operation::Intersection,
            shape_a: "LineSegment2D",
            shape_b: "InfiniteLine2D",
            required: true,
            symmetric: true,
            cardinality: Some(Cardinality::Optional),
            entrypoint_a_to_b: Some("line_segment2d_infinite_line2d_intersection"),
            entrypoint_b_to_a: Some("infinite_line2d_line_segment2d_intersection"),
        },
        MatrixEntry {
            id: "2d:line-ray:intersection",
            dimension: Dimension::D2,
            operation: Operation::Intersection,
            shape_a: "InfiniteLine2D",
            shape_b: "Ray2D",
            required: true,
            symmetric: true,
            cardinality: Some(Cardinality::Optional),
            entrypoint_a_to_b: Some("infinite_line2d_ray2d_intersection"),
            entrypoint_b_to_a: Some("ray2d_infinite_line2d_intersection"),
        },
        MatrixEntry {
            id: "2d:ray-line:intersection",
            dimension: Dimension::D2,
            operation: Operation::Intersection,
            shape_a: "Ray2D",
            shape_b: "InfiniteLine2D",
            required: true,
            symmetric: true,
            cardinality: Some(Cardinality::Optional),
            entrypoint_a_to_b: Some("ray2d_infinite_line2d_intersection"),
            entrypoint_b_to_a: Some("infinite_line2d_ray2d_intersection"),
        },
        MatrixEntry {
            id: "2d:triangle-segment:intersections",
            dimension: Dimension::D2,
            operation: Operation::Intersection,
            shape_a: "Triangle2D",
            shape_b: "LineSegment2D",
            required: true,
            symmetric: false,
            cardinality: Some(Cardinality::Multiple),
            entrypoint_a_to_b: Some("triangle2d_line_segment2d_intersections"),
            entrypoint_b_to_a: None,
        },
        MatrixEntry {
            id: "2d:segment-point:distance",
            dimension: Dimension::D2,
            operation: Operation::Distance,
            shape_a: "LineSegment2D",
            shape_b: "Point2D",
            required: true,
            symmetric: true,
            cardinality: None,
            entrypoint_a_to_b: Some("line_segment2d_point2d_distance"),
            entrypoint_b_to_a: Some("point2d_line_segment2d_distance"),
        },
        MatrixEntry {
            id: "2d:point-segment:distance",
            dimension: Dimension::D2,
            operation: Operation::Distance,
            shape_a: "Point2D",
            shape_b: "LineSegment2D",
            required: true,
            symmetric: true,
            cardinality: None,
            entrypoint_a_to_b: Some("point2d_line_segment2d_distance"),
            entrypoint_b_to_a: Some("line_segment2d_point2d_distance"),
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
        MatrixEntry {
            id: "3d:line-ray:intersection",
            dimension: Dimension::D3,
            operation: Operation::Intersection,
            shape_a: "InfiniteLine3D",
            shape_b: "Ray3D",
            required: true,
            symmetric: true,
            cardinality: Some(Cardinality::Optional),
            entrypoint_a_to_b: Some("infinite_line3d_ray3d_intersection"),
            entrypoint_b_to_a: Some("ray3d_infinite_line3d_intersection"),
        },
        MatrixEntry {
            id: "3d:plane-segment:intersection",
            dimension: Dimension::D3,
            operation: Operation::Intersection,
            shape_a: "Plane3D",
            shape_b: "LineSegment3D",
            required: true,
            symmetric: false,
            cardinality: Some(Cardinality::Optional),
            entrypoint_a_to_b: Some("plane3d_line_segment3d_intersection"),
            entrypoint_b_to_a: None,
        },
        MatrixEntry {
            id: "3d:plane-ray:intersection",
            dimension: Dimension::D3,
            operation: Operation::Intersection,
            shape_a: "Plane3D",
            shape_b: "Ray3D",
            required: true,
            symmetric: false,
            cardinality: Some(Cardinality::Optional),
            entrypoint_a_to_b: Some("plane3d_ray3d_intersection"),
            entrypoint_b_to_a: None,
        },
        MatrixEntry {
            id: "3d:ray-line:intersection",
            dimension: Dimension::D3,
            operation: Operation::Intersection,
            shape_a: "Ray3D",
            shape_b: "InfiniteLine3D",
            required: true,
            symmetric: true,
            cardinality: Some(Cardinality::Optional),
            entrypoint_a_to_b: Some("ray3d_infinite_line3d_intersection"),
            entrypoint_b_to_a: Some("infinite_line3d_ray3d_intersection"),
        },
        MatrixEntry {
            id: "3d:triangle-segment:intersection",
            dimension: Dimension::D3,
            operation: Operation::Intersection,
            shape_a: "Triangle3D",
            shape_b: "LineSegment3D",
            required: true,
            symmetric: true,
            cardinality: Some(Cardinality::Optional),
            entrypoint_a_to_b: Some("triangle3d_line_segment3d_intersection"),
            entrypoint_b_to_a: Some("line_segment3d_triangle3d_intersection"),
        },
        MatrixEntry {
            id: "3d:segment-triangle:intersection",
            dimension: Dimension::D3,
            operation: Operation::Intersection,
            shape_a: "LineSegment3D",
            shape_b: "Triangle3D",
            required: true,
            symmetric: true,
            cardinality: Some(Cardinality::Optional),
            entrypoint_a_to_b: Some("line_segment3d_triangle3d_intersection"),
            entrypoint_b_to_a: Some("triangle3d_line_segment3d_intersection"),
        },
        MatrixEntry {
            id: "3d:triangle-ray:intersection",
            dimension: Dimension::D3,
            operation: Operation::Intersection,
            shape_a: "Triangle3D",
            shape_b: "Ray3D",
            required: true,
            symmetric: true,
            cardinality: Some(Cardinality::Optional),
            entrypoint_a_to_b: Some("triangle3d_ray3d_intersection"),
            entrypoint_b_to_a: Some("ray3d_triangle3d_intersection"),
        },
        MatrixEntry {
            id: "3d:ray-triangle:intersection",
            dimension: Dimension::D3,
            operation: Operation::Intersection,
            shape_a: "Ray3D",
            shape_b: "Triangle3D",
            required: true,
            symmetric: true,
            cardinality: Some(Cardinality::Optional),
            entrypoint_a_to_b: Some("ray3d_triangle3d_intersection"),
            entrypoint_b_to_a: Some("triangle3d_ray3d_intersection"),
        },
        MatrixEntry {
            id: "3d:line-line:intersection",
            dimension: Dimension::D3,
            operation: Operation::Intersection,
            shape_a: "InfiniteLine3D",
            shape_b: "InfiniteLine3D",
            required: true,
            symmetric: false,
            cardinality: Some(Cardinality::Optional),
            entrypoint_a_to_b: Some("infinite_line3d_infinite_line3d_intersection"),
            entrypoint_b_to_a: None,
        },
        MatrixEntry {
            id: "3d:segment-segment:intersection",
            dimension: Dimension::D3,
            operation: Operation::Intersection,
            shape_a: "LineSegment3D",
            shape_b: "LineSegment3D",
            required: true,
            symmetric: false,
            cardinality: Some(Cardinality::Optional),
            entrypoint_a_to_b: Some("line_segment3d_line_segment3d_intersection"),
            entrypoint_b_to_a: None,
        },
        MatrixEntry {
            id: "3d:ray-ray:intersection",
            dimension: Dimension::D3,
            operation: Operation::Intersection,
            shape_a: "Ray3D",
            shape_b: "Ray3D",
            required: true,
            symmetric: false,
            cardinality: Some(Cardinality::Optional),
            entrypoint_a_to_b: Some("ray3d_ray3d_intersection"),
            entrypoint_b_to_a: None,
        },
        MatrixEntry {
            id: "3d:line-sphere:collision",
            dimension: Dimension::D3,
            operation: Operation::Collision,
            shape_a: "InfiniteLine3D",
            shape_b: "SphericalSurface3D",
            required: true,
            symmetric: false,
            cardinality: None,
            entrypoint_a_to_b: Some("infinite_line3d_spherical_surface3d_collides"),
            entrypoint_b_to_a: None,
        },
        MatrixEntry {
            id: "3d:triangle-segment:collision",
            dimension: Dimension::D3,
            operation: Operation::Collision,
            shape_a: "Triangle3D",
            shape_b: "LineSegment3D",
            required: true,
            symmetric: true,
            cardinality: None,
            entrypoint_a_to_b: Some("triangle3d_line_segment3d_collides"),
            entrypoint_b_to_a: Some("line_segment3d_triangle3d_collides"),
        },
        MatrixEntry {
            id: "3d:segment-triangle:collision",
            dimension: Dimension::D3,
            operation: Operation::Collision,
            shape_a: "LineSegment3D",
            shape_b: "Triangle3D",
            required: true,
            symmetric: true,
            cardinality: None,
            entrypoint_a_to_b: Some("line_segment3d_triangle3d_collides"),
            entrypoint_b_to_a: Some("triangle3d_line_segment3d_collides"),
        },
        MatrixEntry {
            id: "3d:triangle-ray:collision",
            dimension: Dimension::D3,
            operation: Operation::Collision,
            shape_a: "Triangle3D",
            shape_b: "Ray3D",
            required: true,
            symmetric: true,
            cardinality: None,
            entrypoint_a_to_b: Some("triangle3d_ray3d_collides"),
            entrypoint_b_to_a: Some("ray3d_triangle3d_collides"),
        },
        MatrixEntry {
            id: "3d:ray-triangle:collision",
            dimension: Dimension::D3,
            operation: Operation::Collision,
            shape_a: "Ray3D",
            shape_b: "Triangle3D",
            required: true,
            symmetric: true,
            cardinality: None,
            entrypoint_a_to_b: Some("ray3d_triangle3d_collides"),
            entrypoint_b_to_a: Some("triangle3d_ray3d_collides"),
        },
        MatrixEntry {
            id: "3d:ray-segment:collision",
            dimension: Dimension::D3,
            operation: Operation::Collision,
            shape_a: "Ray3D",
            shape_b: "LineSegment3D",
            required: true,
            symmetric: true,
            cardinality: None,
            entrypoint_a_to_b: Some("ray3d_line_segment3d_collides"),
            entrypoint_b_to_a: Some("line_segment3d_ray3d_collides"),
        },
        MatrixEntry {
            id: "3d:segment-ray:collision",
            dimension: Dimension::D3,
            operation: Operation::Collision,
            shape_a: "LineSegment3D",
            shape_b: "Ray3D",
            required: true,
            symmetric: true,
            cardinality: None,
            entrypoint_a_to_b: Some("line_segment3d_ray3d_collides"),
            entrypoint_b_to_a: Some("ray3d_line_segment3d_collides"),
        },
        MatrixEntry {
            id: "3d:line-segment:collision",
            dimension: Dimension::D3,
            operation: Operation::Collision,
            shape_a: "InfiniteLine3D",
            shape_b: "LineSegment3D",
            required: true,
            symmetric: true,
            cardinality: None,
            entrypoint_a_to_b: Some("infinite_line3d_line_segment3d_collides"),
            entrypoint_b_to_a: Some("line_segment3d_infinite_line3d_collides"),
        },
        MatrixEntry {
            id: "3d:segment-line:collision",
            dimension: Dimension::D3,
            operation: Operation::Collision,
            shape_a: "LineSegment3D",
            shape_b: "InfiniteLine3D",
            required: true,
            symmetric: true,
            cardinality: None,
            entrypoint_a_to_b: Some("line_segment3d_infinite_line3d_collides"),
            entrypoint_b_to_a: Some("infinite_line3d_line_segment3d_collides"),
        },
        MatrixEntry {
            id: "3d:line-ray:collision",
            dimension: Dimension::D3,
            operation: Operation::Collision,
            shape_a: "InfiniteLine3D",
            shape_b: "Ray3D",
            required: true,
            symmetric: true,
            cardinality: None,
            entrypoint_a_to_b: Some("infinite_line3d_ray3d_collides"),
            entrypoint_b_to_a: Some("ray3d_infinite_line3d_collides"),
        },
        MatrixEntry {
            id: "3d:ray-line:collision",
            dimension: Dimension::D3,
            operation: Operation::Collision,
            shape_a: "Ray3D",
            shape_b: "InfiniteLine3D",
            required: true,
            symmetric: true,
            cardinality: None,
            entrypoint_a_to_b: Some("ray3d_infinite_line3d_collides"),
            entrypoint_b_to_a: Some("infinite_line3d_ray3d_collides"),
        },
        MatrixEntry {
            id: "3d:segment-sphere:intersection",
            dimension: Dimension::D3,
            operation: Operation::Intersection,
            shape_a: "LineSegment3D",
            shape_b: "SphericalSurface3D",
            required: true,
            symmetric: false,
            cardinality: Some(Cardinality::Multiple),
            entrypoint_a_to_b: Some("line_segment3d_spherical_surface3d_intersections"),
            entrypoint_b_to_a: None,
        },
        // ── 2D distance entries ───────────────────────────────────────────────
        MatrixEntry {
            id: "2d:infinite_line-point:distance",
            dimension: Dimension::D2,
            operation: Operation::Distance,
            shape_a: "InfiniteLine2D",
            shape_b: "Point2D",
            required: true,
            symmetric: true,
            cardinality: None,
            entrypoint_a_to_b: Some("infinite_line2d_point2d_distance"),
            entrypoint_b_to_a: Some("point2d_infinite_line2d_distance"),
        },
        MatrixEntry {
            id: "2d:point-infinite_line:distance",
            dimension: Dimension::D2,
            operation: Operation::Distance,
            shape_a: "Point2D",
            shape_b: "InfiniteLine2D",
            required: true,
            symmetric: true,
            cardinality: None,
            entrypoint_a_to_b: Some("point2d_infinite_line2d_distance"),
            entrypoint_b_to_a: Some("infinite_line2d_point2d_distance"),
        },
        MatrixEntry {
            id: "2d:ray-point:distance",
            dimension: Dimension::D2,
            operation: Operation::Distance,
            shape_a: "Ray2D",
            shape_b: "Point2D",
            required: true,
            symmetric: true,
            cardinality: None,
            entrypoint_a_to_b: Some("ray2d_point2d_distance"),
            entrypoint_b_to_a: Some("point2d_ray2d_distance"),
        },
        MatrixEntry {
            id: "2d:point-ray:distance",
            dimension: Dimension::D2,
            operation: Operation::Distance,
            shape_a: "Point2D",
            shape_b: "Ray2D",
            required: true,
            symmetric: true,
            cardinality: None,
            entrypoint_a_to_b: Some("point2d_ray2d_distance"),
            entrypoint_b_to_a: Some("ray2d_point2d_distance"),
        },
        MatrixEntry {
            id: "2d:circle-point:distance",
            dimension: Dimension::D2,
            operation: Operation::Distance,
            shape_a: "Circle2D",
            shape_b: "Point2D",
            required: true,
            symmetric: true,
            cardinality: None,
            entrypoint_a_to_b: Some("circle2d_point2d_distance"),
            entrypoint_b_to_a: Some("point2d_circle2d_distance"),
        },
        MatrixEntry {
            id: "2d:point-circle:distance",
            dimension: Dimension::D2,
            operation: Operation::Distance,
            shape_a: "Point2D",
            shape_b: "Circle2D",
            required: true,
            symmetric: true,
            cardinality: None,
            entrypoint_a_to_b: Some("point2d_circle2d_distance"),
            entrypoint_b_to_a: Some("circle2d_point2d_distance"),
        },
        // ── 3D distance entries ───────────────────────────────────────────────
        MatrixEntry {
            id: "3d:segment-point:distance",
            dimension: Dimension::D3,
            operation: Operation::Distance,
            shape_a: "LineSegment3D",
            shape_b: "Point3D",
            required: true,
            symmetric: true,
            cardinality: None,
            entrypoint_a_to_b: Some("line_segment3d_point3d_distance"),
            entrypoint_b_to_a: Some("point3d_line_segment3d_distance"),
        },
        MatrixEntry {
            id: "3d:point-segment:distance",
            dimension: Dimension::D3,
            operation: Operation::Distance,
            shape_a: "Point3D",
            shape_b: "LineSegment3D",
            required: true,
            symmetric: true,
            cardinality: None,
            entrypoint_a_to_b: Some("point3d_line_segment3d_distance"),
            entrypoint_b_to_a: Some("line_segment3d_point3d_distance"),
        },
        MatrixEntry {
            id: "3d:infinite_line-point:distance",
            dimension: Dimension::D3,
            operation: Operation::Distance,
            shape_a: "InfiniteLine3D",
            shape_b: "Point3D",
            required: true,
            symmetric: true,
            cardinality: None,
            entrypoint_a_to_b: Some("infinite_line3d_point3d_distance"),
            entrypoint_b_to_a: Some("point3d_infinite_line3d_distance"),
        },
        MatrixEntry {
            id: "3d:point-infinite_line:distance",
            dimension: Dimension::D3,
            operation: Operation::Distance,
            shape_a: "Point3D",
            shape_b: "InfiniteLine3D",
            required: true,
            symmetric: true,
            cardinality: None,
            entrypoint_a_to_b: Some("point3d_infinite_line3d_distance"),
            entrypoint_b_to_a: Some("infinite_line3d_point3d_distance"),
        },
        MatrixEntry {
            id: "3d:plane-point:distance",
            dimension: Dimension::D3,
            operation: Operation::Distance,
            shape_a: "Plane3D",
            shape_b: "Point3D",
            required: true,
            symmetric: true,
            cardinality: None,
            entrypoint_a_to_b: Some("plane3d_point3d_distance"),
            entrypoint_b_to_a: Some("point3d_plane3d_distance"),
        },
        MatrixEntry {
            id: "3d:point-plane:distance",
            dimension: Dimension::D3,
            operation: Operation::Distance,
            shape_a: "Point3D",
            shape_b: "Plane3D",
            required: true,
            symmetric: true,
            cardinality: None,
            entrypoint_a_to_b: Some("point3d_plane3d_distance"),
            entrypoint_b_to_a: Some("plane3d_point3d_distance"),
        },
        MatrixEntry {
            id: "3d:ray-point:distance",
            dimension: Dimension::D3,
            operation: Operation::Distance,
            shape_a: "Ray3D",
            shape_b: "Point3D",
            required: true,
            symmetric: true,
            cardinality: None,
            entrypoint_a_to_b: Some("ray3d_point3d_distance"),
            entrypoint_b_to_a: Some("point3d_ray3d_distance"),
        },
        MatrixEntry {
            id: "3d:point-ray:distance",
            dimension: Dimension::D3,
            operation: Operation::Distance,
            shape_a: "Point3D",
            shape_b: "Ray3D",
            required: true,
            symmetric: true,
            cardinality: None,
            entrypoint_a_to_b: Some("point3d_ray3d_distance"),
            entrypoint_b_to_a: Some("ray3d_point3d_distance"),
        },
        MatrixEntry {
            id: "3d:circle-point:distance",
            dimension: Dimension::D3,
            operation: Operation::Distance,
            shape_a: "Circle3D",
            shape_b: "Point3D",
            required: true,
            symmetric: true,
            cardinality: None,
            entrypoint_a_to_b: Some("circle3d_point3d_distance"),
            entrypoint_b_to_a: Some("point3d_circle3d_distance"),
        },
        MatrixEntry {
            id: "3d:point-circle:distance",
            dimension: Dimension::D3,
            operation: Operation::Distance,
            shape_a: "Point3D",
            shape_b: "Circle3D",
            required: true,
            symmetric: true,
            cardinality: None,
            entrypoint_a_to_b: Some("point3d_circle3d_distance"),
            entrypoint_b_to_a: Some("circle3d_point3d_distance"),
        },
    ]
}

#[test]
fn matrix_engine_passes_for_valid_seed() {
    let entries = valid_seed_entries();
    let result = validate_matrix(&entries);
    assert!(result.is_ok(), "{}", result.unwrap_err());
}

#[test]
fn matrix_engine_seed_contains_distance_entries() {
    let entries = valid_seed_entries();
    assert!(
        entries
            .iter()
            .any(|e| matches!(e.operation, Operation::Distance)),
        "distance シードが1件以上必要"
    );
}

#[test]
fn matrix_engine_seed_contains_intersection_entries_for_2d_and_3d() {
    let entries = valid_seed_entries();
    let has_2d = entries.iter().any(|e| {
        matches!(e.operation, Operation::Intersection) && matches!(e.dimension, Dimension::D2)
    });
    let has_3d = entries.iter().any(|e| {
        matches!(e.operation, Operation::Intersection) && matches!(e.dimension, Dimension::D3)
    });

    assert!(has_2d, "2D intersection シードが1件以上必要");
    assert!(has_3d, "3D intersection シードが1件以上必要");
}

#[test]
fn matrix_engine_seed_contains_collision_entries_for_2d_and_3d() {
    let entries = valid_seed_entries();
    let has_2d = entries.iter().any(|e| {
        matches!(e.operation, Operation::Collision) && matches!(e.dimension, Dimension::D2)
    });
    let has_3d = entries.iter().any(|e| {
        matches!(e.operation, Operation::Collision) && matches!(e.dimension, Dimension::D3)
    });

    assert!(has_2d, "2D collision シードが1件以上必要");
    assert!(has_3d, "3D collision シードが1件以上必要");
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

#[test]
fn matrix_engine_fails_for_unknown_entrypoint_symbol() {
    let mut entries = valid_seed_entries();
    entries.push(MatrixEntry {
        id: "2d:unknown:intersection",
        dimension: Dimension::D2,
        operation: Operation::Intersection,
        shape_a: "Circle2D",
        shape_b: "Ray2D",
        required: true,
        symmetric: false,
        cardinality: Some(Cardinality::Single),
        entrypoint_a_to_b: Some("does_not_exist_entrypoint"),
        entrypoint_b_to_a: None,
    });

    let err = validate_matrix(&entries).unwrap_err();
    assert!(err.contains("unknown entrypoints"));
    assert!(err.contains("does_not_exist_entrypoint"));
}

#[test]
fn distance_tolerance_guard_circle2d_point2d() {
    let circle =
        geo_algorithms::Circle2D::new(geo_algorithms::Point2D::new(0.0, 0.0), 1.0).unwrap();
    let on = geo_algorithms::Point2D::new(1.0, 0.0);
    let near = geo_algorithms::Point2D::new(1.0 + 5e-7, 0.0);
    let far = geo_algorithms::Point2D::new(1.0 + 1e-3, 0.0);
    let tol = 1e-6;

    let d_on = geo_algorithms::distance::circle2d_point2d_distance(&circle, &on);
    let d_near = geo_algorithms::distance::circle2d_point2d_distance(&circle, &near);
    let d_far = geo_algorithms::distance::circle2d_point2d_distance(&circle, &far);

    assert!(
        d_on <= tol,
        "on-boundary distance should be within tolerance"
    );
    assert!(
        d_near <= tol,
        "near point distance should be within tolerance"
    );
    assert!(d_far > tol, "far point distance should exceed tolerance");
}

#[test]
fn distance_collision_consistency_for_point_pairs() {
    let tol: f64 = 1e-6;

    let circle =
        geo_algorithms::Circle2D::new(geo_algorithms::Point2D::new(0.0, 0.0), 1.0).unwrap();
    let candidates_2d = [
        geo_algorithms::Point2D::new(1.0, 0.0),
        geo_algorithms::Point2D::new(1.0 + 5e-7, 0.0),
        geo_algorithms::Point2D::new(1.2, 0.0),
    ];

    for point in candidates_2d {
        let dist = geo_algorithms::distance::circle2d_point2d_distance(&circle, &point);
        let collides = geo_algorithms::collision::circle2d_point2d_collides(&circle, &point, tol);
        assert_eq!(
            dist <= tol,
            collides,
            "2D circle-point consistency violated"
        );
    }

    let plane = geo_algorithms::Plane3D::xy_plane(0.0);
    let candidates_3d = [
        geo_algorithms::Point3D::new(0.0, 0.0, 0.0),
        geo_algorithms::Point3D::new(0.0, 0.0, 5e-7),
        geo_algorithms::Point3D::new(0.0, 0.0, 1e-2),
    ];

    for point in candidates_3d {
        let signed_dist: f64 = geo_algorithms::distance::plane3d_point3d_distance(&plane, &point);
        let collides = geo_algorithms::collision::plane3d_point3d_collides(&plane, &point, tol);
        assert_eq!(
            signed_dist.abs() <= tol,
            collides,
            "3D plane-point consistency violated"
        );
    }
}
