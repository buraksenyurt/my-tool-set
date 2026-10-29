//! Domain model for my-tool-set.
//!
//! This crate has no I/O: it describes tools (`tool.toml`), their releases
//! (`release.json`) and the platforms they run on. Reading those files from
//! disk or GitHub, and installing tools, lives in mts-catalog.

pub mod manifest;
mod platform;
pub mod release;
pub mod version;

pub use manifest::{Delivery, HostMode, ToolKind, ToolManifest};
pub use platform::{Arch, Os, Platform};
pub use release::{ArchiveFormat, ReleaseManifest};
pub use version::Version;
