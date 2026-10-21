//! Typed models shared across the IPC boundary.
//!
//! Field names are serialised as camelCase so the TypeScript definitions in
//! `src/ipc/types.ts` can mirror these structs exactly.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Game {
    /// Stable UUID, assigned on import and never reused.
    pub id: String,
    pub title: String,
    /// Canonicalised absolute path to the game executable.
    pub executable_path: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub install_directory: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub provider_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub profile_id: Option<String>,
    /// RFC 3339 timestamps.
    pub created_at: String,
    pub updated_at: String,
}

/// A compatibility capability as *detected on this host*. `available` is an
/// observation, never an inference from a game's name or store listing.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Capability {
    pub id: String,
    pub display_name: String,
    pub available: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,
    /// Why it is unavailable, when it is.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub detail: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Diagnostics {
    pub app_version: String,
    pub os_version: String,
    pub arch: String,
    pub capabilities: Vec<Capability>,
    /// Where the library file lives, so the UI can show it in settings.
    pub library_path: String,
}

/// Whether a game's files are reachable right now. Recomputed on read rather
/// than stored, because an external volume can disappear between calls.
#[derive(Debug, Clone, Copy, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum Availability {
    Available,
    ExecutableMissing,
    VolumeUnavailable,
}

/// A library entry plus its freshly checked availability.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GameEntry {
    #[serde(flatten)]
    pub game: Game,
    pub availability: Availability,
}
