//! 幾何 foundation capability の公開 facade です。
//!
//! foundation 配下の実体は責務別 module に分割し、ここでは公開面だけを束ねます。

pub mod bounds;
pub mod metadata;

pub use bounds::Bounded;
pub use metadata::PrimitiveMetadata;
