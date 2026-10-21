//! Structured errors crossing the IPC boundary.
//!
//! Every command returns `Result<T, IpcError>`. The UI switches on `code`,
//! which is stable, and displays `message`, which is safe to show a user.
//! `detail` carries diagnostic text that may be verbose and is never required.

use serde::Serialize;

/// Stable, machine-readable error codes. The UI may branch on these, so a
/// variant's string is part of the IPC contract and must not change.
pub mod code {
    pub const PATH_NOT_ABSOLUTE: &str = "PATH_NOT_ABSOLUTE";
    pub const PATH_NOT_FOUND: &str = "PATH_NOT_FOUND";
    pub const PATH_NOT_A_FILE: &str = "PATH_NOT_A_FILE";
    pub const PATH_INVALID: &str = "PATH_INVALID";
    pub const PATH_UNREADABLE: &str = "PATH_UNREADABLE";
    pub const VOLUME_UNAVAILABLE: &str = "VOLUME_UNAVAILABLE";
    pub const GAME_NOT_FOUND: &str = "GAME_NOT_FOUND";
    pub const GAME_ALREADY_IMPORTED: &str = "GAME_ALREADY_IMPORTED";
    pub const STORE_UNREADABLE: &str = "STORE_UNREADABLE";
    pub const STORE_UNWRITABLE: &str = "STORE_UNWRITABLE";
    pub const STORE_CORRUPT: &str = "STORE_CORRUPT";
    pub const INTERNAL: &str = "INTERNAL";
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct IpcError {
    pub code: String,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub detail: Option<String>,
}

impl IpcError {
    pub fn new(code: &str, message: impl Into<String>) -> Self {
        Self {
            code: code.to_string(),
            message: message.into(),
            detail: None,
        }
    }

    pub fn detailed(code: &str, message: impl Into<String>, detail: impl Into<String>) -> Self {
        Self {
            code: code.to_string(),
            message: message.into(),
            detail: Some(detail.into()),
        }
    }

    pub fn internal(detail: impl Into<String>) -> Self {
        Self::detailed(
            code::INTERNAL,
            "Something went wrong inside ReaPlay.",
            detail,
        )
    }
}

impl std::fmt::Display for IpcError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: {}", self.code, self.message)
    }
}

impl std::error::Error for IpcError {}
