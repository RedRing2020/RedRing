//! Distance operation contracts.
//!
//! Cross-shape distance contracts are defined in `geometry::operations`
//! and implemented in higher-level algorithm crates.

use analysis::abstract_types::Scalar;

/// Cross-shape distance contract (deterministic).
pub trait CrossDistance<T: Scalar, Other> {
    /// Computes the shortest distance to `other`.
    fn distance_to(&self, other: &Other) -> T;
}

/// Standardized failure for iterative distance solvers.
#[derive(Debug, Clone, PartialEq)]
pub enum DistanceConvergenceError<T: Scalar> {
    /// Iterative solver did not converge within the configured budget.
    NotConverged { iterations: usize, residual: T },
    /// Initial parameters were invalid for the target domain.
    InvalidInitialization,
    /// Numerical state became invalid during the iteration.
    NumericalFailure,
}
