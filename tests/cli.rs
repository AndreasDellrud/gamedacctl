use std::{fs, process::Command};

use serde_json::Value;
use tempfile::tempdir;

fn gamedacctl() -> Command {
    Command::new(env!("CARGO_BIN_EXE_gamedacctl"))
}

fn profile_store(config_home: &std::path::Path) {
    let path = config_home.join("gamedacctl/profiles.json");
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(
        path,
        r##"{
  "schema_version": 1,
  "last_selected": "Everyday",
  "apply_on_reconnect": true,
  "profiles": [
    {
      "name": "Everyday",
      "icon": "💜",
      "lighting": {
        "effect": "breathe",
        "color": "7A21E6",
        "seconds": 10,
        "mode": "synchronized",
        "reverse": false
      }
    },
    {
      "name": "Legacy",
      "lighting": {
        "effect": "static",
        "left": "FF3700",
        "right": "0084FF",
        "microphone_live": "00FF00",
        "microphone_muted": "FF0000"
      }
    }
  ]
}"##,
    )
    .unwrap();
}

#[test]
fn status_json_reports_profiles_and_a_machine_readable_device_state() {
    let config_home = tempdir().unwrap();
    profile_store(config_home.path());
    let output = gamedacctl()
        .env("XDG_CONFIG_HOME", config_home.path())
        .args(["status", "--json"])
        .output()
        .unwrap();

    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let response: Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(response["schema_version"], 1);
    assert!(response["device"]["state"].is_string());
    assert_eq!(response["apply_on_reconnect"], true);
    assert_eq!(response["lighting_enabled"], true);
    assert_eq!(response["profiles"][0]["name"], "Everyday");
    assert_eq!(response["profiles"][0]["icon"], "💜");
    assert_eq!(response["profiles"][0]["selected"], true);
    assert_eq!(response["profiles"][0]["effect"], "breathe");
    assert_eq!(response["profiles"][1]["name"], "Legacy");
    assert!(response["profiles"][1]["icon"].is_null());
}

#[test]
fn saved_profile_apply_dry_run_does_not_open_hid_or_rewrite_store() {
    let config_home = tempdir().unwrap();
    profile_store(config_home.path());
    let path = config_home.path().join("gamedacctl/profiles.json");
    let before = fs::read(&path).unwrap();
    let output = gamedacctl()
        .env("XDG_CONFIG_HOME", config_home.path())
        .args(["--dry-run", "profile", "apply", "Everyday", "--json"])
        .output()
        .unwrap();

    assert!(output.status.success());
    assert!(
        String::from_utf8(output.stdout)
            .unwrap()
            .contains("feature zone=right")
    );
    assert_eq!(fs::read(path).unwrap(), before);
}

#[test]
fn missing_saved_profile_fails_without_opening_hid() {
    let config_home = tempdir().unwrap();
    profile_store(config_home.path());
    let output = gamedacctl()
        .env("XDG_CONFIG_HOME", config_home.path())
        .args(["--dry-run", "profile", "apply", "Missing"])
        .output()
        .unwrap();

    assert!(!output.status.success());
    assert!(
        String::from_utf8(output.stderr)
            .unwrap()
            .contains("was not found")
    );
}

#[test]
fn master_lighting_off_dry_run_covers_every_zone_without_rewriting_store() {
    let config_home = tempdir().unwrap();
    profile_store(config_home.path());
    let path = config_home.path().join("gamedacctl/profiles.json");
    let before = fs::read(&path).unwrap();
    let output = gamedacctl()
        .env("XDG_CONFIG_HOME", config_home.path())
        .args(["--dry-run", "profile", "lighting", "off", "--json"])
        .output()
        .unwrap();

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("feature zone=left length=1024 bytes=AA 00 00 00 00"));
    assert!(stdout.contains("feature zone=right length=1024 bytes=AA 01 00 00 00"));
    assert!(stdout.contains("feature zone=microphone-live"));
    assert!(stdout.contains("feature zone=microphone-muted"));
    assert!(stdout.contains("zone-mask=0x0F"));
    assert_eq!(fs::read(path).unwrap(), before);
}

#[test]
fn master_lighting_on_dry_run_restores_the_selected_profile() {
    let config_home = tempdir().unwrap();
    profile_store(config_home.path());
    let output = gamedacctl()
        .env("XDG_CONFIG_HOME", config_home.path())
        .args(["--dry-run", "profile", "lighting", "on", "--json"])
        .output()
        .unwrap();

    assert!(output.status.success());
    assert!(
        String::from_utf8(output.stdout)
            .unwrap()
            .contains("feature zone=right")
    );
}

#[test]
fn static_dry_run_reports_all_selected_zones_without_opening_hid() {
    let output = gamedacctl()
        .args([
            "--dry-run",
            "static",
            "--left",
            "ff3700",
            "--right",
            "0084ff",
            "--microphone-live",
            "00ff00",
            "--microphone-muted",
            "ff0000",
        ])
        .output()
        .unwrap();

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("feature zone=left length=1024 bytes=AA 00 FF 37 00"));
    assert!(stdout.contains("feature zone=right length=1024 bytes=AA 01 00 84 FF"));
    assert!(stdout.contains("feature zone=microphone-live"));
    assert!(stdout.contains("feature zone=microphone-muted"));
    assert!(stdout.contains("output length=64 bytes=A5 0F 0A"));
    assert!(stdout.contains("zone-mask=0x0F"));
}

#[test]
fn empty_static_command_fails_closed() {
    let output = gamedacctl().args(["--dry-run", "static"]).output().unwrap();

    assert!(!output.status.success());
    assert!(
        String::from_utf8(output.stderr)
            .unwrap()
            .contains("at least one lighting zone must be selected")
    );
}

#[test]
fn invalid_color_is_rejected_by_argument_parser() {
    let output = gamedacctl()
        .args(["--dry-run", "static", "--left", "not-a-color"])
        .output()
        .unwrap();

    assert_eq!(output.status.code(), Some(2));
    assert!(
        String::from_utf8(output.stderr)
            .unwrap()
            .contains("color must contain exactly six hexadecimal digits")
    );
}

#[test]
fn off_defaults_to_earcups_only() {
    let output = gamedacctl().args(["--dry-run", "off"]).output().unwrap();

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("feature zone=left"));
    assert!(stdout.contains("feature zone=right"));
    assert!(!stdout.contains("feature zone=microphone"));
    assert!(stdout.contains("zone-mask=0x03"));
}

#[test]
fn breathe_dry_run_generates_both_earcups_and_connected_fields() {
    let output = gamedacctl()
        .args([
            "--dry-run",
            "breathe",
            "--color",
            "2468ac",
            "--seconds",
            "10",
            "--mode",
            "sweep",
            "--reverse",
        ])
        .output()
        .unwrap();

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("feature zone=right length=1024 bytes=AA 01 24 68 AC"));
    assert!(stdout.contains("feature zone=left length=1024 bytes=AA 00 24 68 AC"));
    assert!(stdout.contains("FF FD FB 00 F4 01 01 00 01 03 05 00 F4 01"));
    assert!(stdout.contains("output length=64 bytes=A5 03 0A"));
    assert!(stdout.contains("zone-mask=0x03"));
}

#[test]
fn breathe_rejects_unverified_durations_and_invalid_reverse_mode() {
    for seconds in ["0", "31"] {
        let output = gamedacctl()
            .args([
                "--dry-run",
                "breathe",
                "--color",
                "123456",
                "--seconds",
                seconds,
            ])
            .output()
            .unwrap();
        assert!(!output.status.success());
        assert!(
            String::from_utf8(output.stderr)
                .unwrap()
                .contains("whole number from 1 through 30 seconds")
        );
    }

    let output = gamedacctl()
        .args([
            "--dry-run",
            "breathe",
            "--color",
            "123456",
            "--seconds",
            "5",
            "--reverse",
        ])
        .output()
        .unwrap();
    assert!(!output.status.success());
    assert!(
        String::from_utf8(output.stderr)
            .unwrap()
            .contains("reverse direction is observed only")
    );
}

#[test]
fn color_shift_dry_run_accepts_exactly_two_colors() {
    let output = gamedacctl()
        .args([
            "--dry-run",
            "color-shift",
            "--color",
            "FF0000",
            "--color",
            "0000FF",
            "--seconds",
            "10",
        ])
        .output()
        .unwrap();

    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("feature zone=right length=1024 bytes=AA 01 FF 00 00"));
    assert!(stdout.contains("F8 00 08 00 F4 01 01 00 08 00 F8 00 F4 01"));
    assert!(stdout.contains("zone-mask=0x03"));
}

#[test]
fn multi_color_breathe_dry_run_accepts_four_colors_and_rejects_five() {
    let accepted = gamedacctl()
        .args([
            "--dry-run",
            "multi-color-breathe",
            "--color",
            "FF0000",
            "--color",
            "FFFF00",
            "--color",
            "00FF00",
            "--color",
            "0000FF",
            "--seconds",
            "10",
        ])
        .output()
        .unwrap();
    assert!(
        accepted.status.success(),
        "{}",
        String::from_utf8_lossy(&accepted.stderr)
    );
    assert!(
        String::from_utf8(accepted.stdout)
            .unwrap()
            .contains("zone-mask=0x03")
    );

    let rejected = gamedacctl()
        .args([
            "--dry-run",
            "multi-color-breathe",
            "--color",
            "FF0000",
            "--color",
            "FFFF00",
            "--color",
            "00FF00",
            "--color",
            "00FFFF",
            "--color",
            "0000FF",
            "--seconds",
            "10",
        ])
        .output()
        .unwrap();
    assert_eq!(rejected.status.code(), Some(1));
    assert!(
        String::from_utf8(rejected.stderr)
            .unwrap()
            .contains("requires between 1 and 4 colors; got 5")
    );
}

#[test]
fn passive_input_observation_rejects_dry_run_without_opening_hid() {
    let output = gamedacctl()
        .args(["--dry-run", "observe-input", "--seconds", "0"])
        .output()
        .unwrap();

    assert!(!output.status.success());
    assert!(
        String::from_utf8(output.stderr)
            .unwrap()
            .contains("--dry-run is not meaningful for passive input observation")
    );
}
