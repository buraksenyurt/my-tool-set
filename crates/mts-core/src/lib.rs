//! Domain model for my-tool-set.
//!
//! This crate has no I/O: it only describes tools and the platforms they run on.
//! Reading manifests and releases lives in `mts-catalog` (phase 1), starting and
//! watching processes in `mts-runner` (phase 3).

mod platform;

pub use platform::{Arch, Os, Platform};
