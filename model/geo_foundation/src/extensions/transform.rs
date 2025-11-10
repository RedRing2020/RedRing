//! Basic transformation traits
//!
//! This module defines abstract transformation interfaces that can be implemented
//! by geometric primitives. These traits use associated types to remain abstract
//! about the specific geometric types involved.

use crate::Scalar;

/// Basic 2D transformation operations
///
/// This trait defines the fundamental transformation operations that 2D geometric
/// primitives should support. It uses associated types to remain abstract about
/// the specific point, vector, and angle types.
pub trait BasicTransform<T: Scalar> {
    /// Vector type for translations
    type Vector2D;
    /// Point type for rotation and scaling centers
    type Point2D;
    /// Angle type for rotations
    type Angle: Copy;
    /// The type returned after transformation
    type Transformed;

    /// Translate the primitive by a vector
    fn translate(&self, translation: Self::Vector2D) -> Self::Transformed;

    /// Rotate the primitive around a center point by an angle
    fn rotate(&self, center: Self::Point2D, angle: Self::Angle) -> Self::Transformed;

    /// Scale the primitive from a center point by a uniform factor
    fn scale(&self, center: Self::Point2D, factor: T) -> Self::Transformed;
}

/// Basic 3D transformation operations
///
/// This trait defines the fundamental transformation operations that 3D geometric
/// primitives should support.
pub trait BasicTransform3D<T: Scalar> {
    /// Vector type for 3D translations
    type Vector3D;
    /// Point type for rotation and scaling centers
    type Point3D;
    /// Rotation type (could be quaternion, axis-angle, etc.)
    type Rotation3D;
    /// The type returned after transformation
    type Transformed;

    /// Translate the primitive by a 3D vector
    fn translate_3d(&self, translation: Self::Vector3D) -> Self::Transformed;

    /// Rotate the primitive around a center point with 3D rotation
    fn rotate_3d(&self, center: Self::Point3D, rotation: Self::Rotation3D) -> Self::Transformed;

    /// Scale the primitive from a center point by a uniform factor in 3D
    fn scale_3d(&self, center: Self::Point3D, factor: T) -> Self::Transformed;
}

/// Advanced 2D transformation operations
///
/// This trait extends BasicTransform with more sophisticated operations
/// like mirroring and matrix transformations.
pub trait AdvancedTransform<T: Scalar>: BasicTransform<T> {
    /// Line type for mirroring operations
    type Line2D;
    /// Matrix type for general transformations
    type Matrix3;

    /// Mirror the primitive across a line
    fn mirror(&self, axis: Self::Line2D) -> Self::Transformed;

    /// Scale the primitive with different factors for x and y
    fn non_uniform_scale(&self, center: Self::Point2D, scale_x: T, scale_y: T)
        -> Self::Transformed;

    /// Apply a general transformation matrix
    fn transform_matrix(&self, matrix: &Self::Matrix3) -> Self::Transformed;

    /// Reverse the orientation of the primitive (e.g., flip curve direction)
    fn reverse(&self) -> Self::Transformed;
}
