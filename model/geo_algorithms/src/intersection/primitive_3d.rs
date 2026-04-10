//! 3D Primitive intersection algorithms

mod circular_family;
mod cylindrical_and_conical_family;
mod linear_family;
mod planar_and_mesh_family;
mod shared;
mod spherical_and_quadric_family;

pub use circular_family::*;
pub use cylindrical_and_conical_family::*;
pub use linear_family::*;
pub use planar_and_mesh_family::*;
pub use spherical_and_quadric_family::*;

#[cfg(test)]
mod tests;
