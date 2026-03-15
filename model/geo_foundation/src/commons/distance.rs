//! Distance helpers for geometry-layer traits.

use crate::Scalar;

/// Compute distance from a point to a 2D ellipse boundary.
pub fn ellipse_2d_distance_to_point<T: Scalar>(
    point_x: T,
    point_y: T,
    semi_major: T,
    semi_minor: T,
) -> T {
    let x_norm = point_x / semi_major;
    let y_norm = point_y / semi_minor;
    let normalized_distance = (x_norm * x_norm + y_norm * y_norm).sqrt();

    if normalized_distance <= T::ONE {
        T::ZERO
    } else {
        let scale = T::ONE / normalized_distance;
        let boundary_x = point_x * scale;
        let boundary_y = point_y * scale;
        ((point_x - boundary_x) * (point_x - boundary_x)
            + (point_y - boundary_y) * (point_y - boundary_y))
            .sqrt()
    }
}

/// Compute 3D point distance against an ellipse on a local plane.
pub fn ellipse_3d_distance_to_point<T: Scalar>(
    local_x: T,
    local_y: T,
    normal_distance: T,
    semi_major: T,
    semi_minor: T,
) -> T {
    let planar_distance = ellipse_2d_distance_to_point(local_x, local_y, semi_major, semi_minor);
    (planar_distance * planar_distance + normal_distance * normal_distance).sqrt()
}

/// Compute shortest distance between a line segment and an AABB.
pub fn line_segment_to_aabb_distance<T: Scalar>(
    segment_start: (T, T, T),
    segment_end: (T, T, T),
    aabb_min: (T, T, T),
    aabb_max: (T, T, T),
) -> T {
    let (sx, sy, sz) = segment_start;
    let (ex, ey, ez) = segment_end;
    let (min_x, min_y, min_z) = aabb_min;
    let (max_x, max_y, max_z) = aabb_max;

    let point_in_aabb = |px: T, py: T, pz: T| -> bool {
        px >= min_x && px <= max_x && py >= min_y && py <= max_y && pz >= min_z && pz <= max_z
    };
    let clamp_to_aabb = |px: T, py: T, pz: T| -> (T, T, T) {
        (
            px.clamp(min_x, max_x),
            py.clamp(min_y, max_y),
            pz.clamp(min_z, max_z),
        )
    };
    let distance = |p1x: T, p1y: T, p1z: T, p2x: T, p2y: T, p2z: T| -> T {
        let dx = p1x - p2x;
        let dy = p1y - p2y;
        let dz = p1z - p2z;
        (dx * dx + dy * dy + dz * dz).sqrt()
    };

    if point_in_aabb(sx, sy, sz) || point_in_aabb(ex, ey, ez) {
        return T::ZERO;
    }

    let dx = ex - sx;
    let dy = ey - sy;
    let dz = ez - sz;
    let num_samples = 10;
    let mut min_distance = T::INFINITY;

    for i in 0..=num_samples {
        let t = T::from_usize(i) / T::from_usize(num_samples);
        let px = sx + dx * t;
        let py = sy + dy * t;
        let pz = sz + dz * t;
        let (cx, cy, cz) = clamp_to_aabb(px, py, pz);
        let dist = distance(px, py, pz, cx, cy, cz);
        min_distance = min_distance.min(dist);
        if dist <= T::EPSILON {
            return T::ZERO;
        }
    }

    let vertices = [
        (min_x, min_y, min_z),
        (max_x, min_y, min_z),
        (min_x, max_y, min_z),
        (max_x, max_y, min_z),
        (min_x, min_y, max_z),
        (max_x, min_y, max_z),
        (min_x, max_y, max_z),
        (max_x, max_y, max_z),
    ];

    for &(vx, vy, vz) in &vertices {
        let to_vx = vx - sx;
        let to_vy = vy - sy;
        let to_vz = vz - sz;
        let dot = to_vx * dx + to_vy * dy + to_vz * dz;
        let len_sq = dx * dx + dy * dy + dz * dz;
        let t = if len_sq <= T::EPSILON {
            T::ZERO
        } else {
            (dot / len_sq).clamp(T::ZERO, T::ONE)
        };
        let closest_x = sx + dx * t;
        let closest_y = sy + dy * t;
        let closest_z = sz + dz * t;
        let dist = distance(vx, vy, vz, closest_x, closest_y, closest_z);
        min_distance = min_distance.min(dist);
    }

    min_distance
}
