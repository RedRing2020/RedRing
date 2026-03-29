//! application - Application Layer orchestration entrypoints
//!
//! This crate is a single receiver for orchestration responsibilities during PoC.
//! The initial phase keeps modules in one crate and promotes them only when boundaries stabilize.

pub mod cam_orchestration;
pub mod feature_orchestration;
pub mod geometry_orchestration;
pub mod job_orchestration;
