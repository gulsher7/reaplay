//! The diagnostic export.
//!
//! A plain-text report a user can paste into a bug report. Home directories
//! are replaced with `~` so a username does not travel with it, and nothing
//! that could hold a secret is included: the report is assembled from host
//! facts, detection results and launch metadata only.

use crate::launch::{LaunchRecord, LaunchStatus};
use crate::model::{Availability, Diagnostics, GameEntry};
use crate::provider::ProviderInfo;

/// Replace the user's home directory with `~`.
///
/// This is the only personal data that routinely appears in ReaPlay's paths.
/// External volume names are left alone: they are needed to make sense of a
/// `VOLUME_UNAVAILABLE` report, and the user chose them.
pub fn redact(path: &str) -> String {
    match std::env::var("HOME") {
        Ok(home) if !home.is_empty() && path.starts_with(&home) => {
            format!("~{}", &path[home.len()..])
        }
        _ => path.to_string(),
    }
}

fn availability_label(availability: Availability) -> &'static str {
    match availability {
        Availability::Available => "available",
        Availability::ExecutableMissing => "executable missing",
        Availability::VolumeUnavailable => "volume unavailable",
    }
}

fn status_label(status: LaunchStatus) -> &'static str {
    match status {
        LaunchStatus::Running => "running",
        LaunchStatus::Exited => "exited",
        LaunchStatus::Failed => "failed",
        LaunchStatus::Terminated => "stopped",
    }
}

pub fn build(
    diagnostics: &Diagnostics,
    providers: &[ProviderInfo],
    games: &[GameEntry],
    launches: &[LaunchRecord],
) -> String {
    let mut out = String::new();

    out.push_str("# ReaPlay diagnostic report\n\n");
    out.push_str(&format!(
        "Generated: {}\n\n",
        chrono::Utc::now().to_rfc3339()
    ));

    out.push_str("## Host\n\n");
    out.push_str(&format!("- macOS: {}\n", diagnostics.os_version));
    out.push_str(&format!("- Architecture: {}\n", diagnostics.arch));
    out.push_str(&format!("- ReaPlay: {}\n", diagnostics.app_version));
    out.push_str(&format!(
        "- Library file: {}\n\n",
        redact(&diagnostics.library_path)
    ));

    out.push_str("## Detected tools\n\n");
    if diagnostics.capabilities.is_empty() {
        out.push_str("None.\n\n");
    } else {
        for capability in &diagnostics.capabilities {
            let mark = if capability.available { "yes" } else { "no" };
            out.push_str(&format!("- {}: {mark}", capability.display_name));
            if let Some(detail) = &capability.detail {
                out.push_str(&format!(" ({})", redact(detail)));
            }
            out.push('\n');
        }
        out.push('\n');
    }

    out.push_str("## Compatibility providers\n\n");
    for provider in providers {
        let mark = if provider.available {
            "available"
        } else {
            "unavailable"
        };
        out.push_str(&format!(
            "- {} ({}): {mark}",
            provider.display_name, provider.id
        ));
        if let Some(path) = &provider.runtime_path {
            out.push_str(&format!(" at {}", redact(path)));
        }
        out.push('\n');
    }
    out.push('\n');

    out.push_str(&format!("## Library ({} game(s))\n\n", games.len()));
    for entry in games {
        out.push_str(&format!(
            "- {} — {} — {}\n",
            entry.game.title,
            availability_label(entry.availability),
            redact(&entry.game.executable_path)
        ));
    }
    if games.is_empty() {
        out.push_str("Empty.\n");
    }
    out.push('\n');

    out.push_str("## Recent launches\n\n");
    if launches.is_empty() {
        out.push_str("None.\n");
    } else {
        // Enough to see a pattern without pasting an essay into a bug report.
        for record in launches.iter().take(10) {
            out.push_str(&format!(
                "- {} — {} — via {} — {}",
                record.started_at,
                record.game_title,
                record.provider_id,
                status_label(record.status)
            ));
            if let Some(exit) = record.exit_code {
                out.push_str(&format!(" (exit {exit})"));
            }
            out.push('\n');
            out.push_str(&format!("  program: {}\n", redact(&record.program)));
            if !record.args.is_empty() {
                let args: Vec<String> = record.args.iter().map(|a| redact(a)).collect();
                out.push_str(&format!("  args: {}\n", args.join(" | ")));
            }
        }
    }

    out.push_str(
        "\n---\nPaths under your home directory are shown as `~`. This report \
         contains no credentials or tokens; ReaPlay never stores any.\n",
    );

    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::Capability;

    fn diagnostics(library_path: &str) -> Diagnostics {
        Diagnostics {
            app_version: "0.1.0".into(),
            os_version: "26.6.2".into(),
            arch: "aarch64".into(),
            capabilities: vec![Capability {
                id: "wine".into(),
                display_name: "Wine".into(),
                available: false,
                version: None,
                detail: Some("Not detected.".into()),
            }],
            library_path: library_path.into(),
        }
    }

    #[test]
    fn a_home_path_is_replaced_with_a_tilde() {
        let home = std::env::var("HOME").expect("HOME set");
        let redacted = redact(&format!("{home}/Library/Application Support/x.json"));
        assert_eq!(redacted, "~/Library/Application Support/x.json");
    }

    #[test]
    fn a_path_outside_home_is_untouched() {
        assert_eq!(redact("/Volumes/T7/Games/g.exe"), "/Volumes/T7/Games/g.exe");
    }

    #[test]
    fn the_report_never_contains_the_home_directory() {
        let home = std::env::var("HOME").expect("HOME set");
        let report = build(
            &diagnostics(&format!(
                "{home}/Library/Application Support/reaplay/library.json"
            )),
            &[ProviderInfo {
                id: "wine".into(),
                display_name: "Wine".into(),
                available: true,
                runtime_path: Some(format!("{home}/.local/bin/wine")),
                detail: None,
            }],
            &[],
            &[],
        );
        assert!(
            !report.contains(&home),
            "the diagnostic export leaked the home directory"
        );
        assert!(report.contains('~'), "paths should be shown relative to ~");
    }

    #[test]
    fn the_report_states_the_host_facts() {
        let report = build(&diagnostics("/tmp/library.json"), &[], &[], &[]);
        assert!(report.contains("26.6.2"));
        assert!(report.contains("aarch64"));
        assert!(report.contains("0.1.0"));
        assert!(report.contains("Empty."));
    }
}
