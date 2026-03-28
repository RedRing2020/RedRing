use super::{Dimension, MatrixEntry, Operation};

pub(super) fn entries() -> Vec<MatrixEntry> {
    vec![
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
    ]
}
