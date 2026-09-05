use std::process::Command;

fn gamedacctl() -> Command {
    Command::new(env!("CARGO_BIN_EXE_gamedacctl"))
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
