//! Shapes sent to the front end. Generated as TypeScript into src/lib/bindings
//! by `cargo test` (ts-rs), so both sides always agree.

use mts_core::Platform;
use serde::Serialize;
use ts_rs::TS;

#[derive(Debug, Clone, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct AppInfoDto {
    pub name: String,
    pub version: String,
    /// `None` when the shell runs on a platform the tools are never released for.
    pub platform: Option<Platform>,
    /// Where installed tools and settings will live.
    pub data_dir: String,
}
