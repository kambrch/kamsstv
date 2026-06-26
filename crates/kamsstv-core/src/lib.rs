//! kamsstv-core — pure, I/O-free SSTV codec core.
//!
//! No audio, hardware, or channel dependencies live here (enforced by this
//! crate's `Cargo.toml`). Shared primitives sit at the root; the asymmetric
//! TX/RX pipelines live in subtrees. See docs/superpowers/specs/.

pub mod color;
pub mod conductor;
pub mod consts;
pub mod image;
pub mod mode;
pub mod tx;
