//! Everything that touches the disk or the network on behalf of the shell:
//!
//! - [`catalog`]: reads `tool.toml` files (embedded, repository, user folder)
//! - [`settings`]: `settings.toml` and resolving local tool folders
//! - [`releases`]: fetches `release.json` and assets from GitHub Releases
//! - [`cache`]: remembers the last `release.json` per tool, for offline starts
//! - [`install`]: downloads, verifies (size + SHA256), unpacks and lists installs
//! - [`markdown`]: README → sanitized HTML with rewritten URLs
//! - [`docs`]: fetches and caches README files and their images

pub mod cache;
pub mod catalog;
pub mod docs;
mod error;
pub mod install;
pub mod markdown;
pub mod releases;
pub mod settings;

pub use error::CatalogError;
