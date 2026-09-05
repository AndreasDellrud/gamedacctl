use std::{path::Path, process::Command};

use thiserror::Error;

use crate::{FeatureReport, LightingPlan, ProtocolError};

#[derive(Debug, Error)]
pub enum CaptureError {
    #[error("capture does not exist or is not a file: {0}")]
    MissingCapture(String),
    #[error("frame numbers must be greater than zero")]
    InvalidFrame,
    #[error("could not execute tshark: {0}")]
    Tshark(#[source] std::io::Error),
    #[error("tshark could not extract frame {frame}: {message}")]
    Extraction { frame: u32, message: String },
    #[error("frame {frame} yielded {count} complete feature reports; expected exactly one")]
    UnexpectedReportCount { frame: u32, count: usize },
    #[error("frame {frame} contains invalid hexadecimal feature data")]
    InvalidHex { frame: u32 },
    #[error("frame {frame} is not a supported GameDAC lighting report: {source}")]
    InvalidReport {
        frame: u32,
        #[source]
        source: ProtocolError,
    },
    #[error(transparent)]
    Protocol(#[from] ProtocolError),
}

pub fn extract_plan(path: &Path, frames: &[u32]) -> Result<LightingPlan, CaptureError> {
    if !path.is_file() {
        return Err(CaptureError::MissingCapture(path.display().to_string()));
    }
    if frames.is_empty() || frames.contains(&0) {
        return Err(CaptureError::InvalidFrame);
    }

    let reports = frames
        .iter()
        .map(|frame| extract_report(path, *frame))
        .collect::<Result<Vec<_>, _>>()?;
    Ok(LightingPlan::captured(reports)?)
}

fn extract_report(path: &Path, frame: u32) -> Result<FeatureReport, CaptureError> {
    let output = Command::new("tshark")
        .args([
            "-r",
            &path.display().to_string(),
            "-Y",
            &format!(
                "frame.number == {frame} && usb.data_len == 1024 && usb.bmRequestType == 0x21"
            ),
            "-T",
            "fields",
            "-e",
            "usb.data_fragment",
        ])
        .output()
        .map_err(CaptureError::Tshark)?;

    if !output.status.success() {
        return Err(CaptureError::Extraction {
            frame,
            message: String::from_utf8_lossy(&output.stderr).trim().to_owned(),
        });
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let values: Vec<_> = stdout
        .lines()
        .filter(|line| !line.trim().is_empty())
        .collect();
    if values.len() != 1 {
        return Err(CaptureError::UnexpectedReportCount {
            frame,
            count: values.len(),
        });
    }

    let compact: String = values[0]
        .chars()
        .filter(|character| !character.is_ascii_whitespace() && *character != ':')
        .collect();
    if compact.len() % 2 != 0 || !compact.bytes().all(|byte| byte.is_ascii_hexdigit()) {
        return Err(CaptureError::InvalidHex { frame });
    }
    let bytes = (0..compact.len())
        .step_by(2)
        .map(|offset| u8::from_str_radix(&compact[offset..offset + 2], 16))
        .collect::<Result<Vec<_>, _>>()
        .map_err(|_| CaptureError::InvalidHex { frame })?;
    FeatureReport::captured(&bytes).map_err(|source| CaptureError::InvalidReport { frame, source })
}

#[cfg(test)]
mod tests {
    use std::path::{Path, PathBuf};

    use sha2::{Digest, Sha256};

    use super::*;
    use crate::{BreatheDuration, BreatheMode, Color, FeatureReport, Zone};

    fn capture(name: &str) -> PathBuf {
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("docs/raw")
            .join(name)
    }

    fn sha256(bytes: &[u8]) -> String {
        format!("{:x}", Sha256::digest(bytes))
    }

    #[test]
    fn extracts_complete_sweep_and_synchronized_fixtures_exactly() {
        let path = capture("capture-connected-modes-20260905.pcapng");

        let sweep = extract_plan(&path, &[7, 11]).unwrap();
        assert_eq!(sweep.zone_mask(), 0x03);
        assert_eq!(sweep.features()[0].zone(), Zone::Right);
        assert_eq!(sweep.features()[1].zone(), Zone::Left);
        assert_eq!(sweep.features()[0].bytes()[152], 1);
        assert_eq!(sweep.features()[1].bytes()[152], 1);
        assert_eq!(
            sha256(sweep.features()[0].bytes()),
            "21aca07422a1fd49381b3188d04ed08873e3c118b99245ca507b793f9a2b5fdf"
        );
        assert_eq!(
            sha256(sweep.features()[1].bytes()),
            "118fea93d213c39406d87cd63f444a13a64b23c1ea2bee52fe1df1868293d35a"
        );

        let synchronized = extract_plan(&path, &[31, 33]).unwrap();
        assert_eq!(synchronized.features()[0].bytes()[152], 0);
        assert_eq!(synchronized.features()[1].bytes()[152], 0);
        assert_eq!(
            sha256(synchronized.features()[0].bytes()),
            "f3162db34bc2db09a05d0e93acf1ff44989287f8eac46c35e03a48e70d2b7cb9"
        );
        assert_eq!(
            sha256(synchronized.features()[1].bytes()),
            "c2279b2d20ee885f3f04adb9e03d03aa26275b9f47a1afc41faac52c318d7e02"
        );
    }

    #[test]
    fn extracts_physically_verified_five_second_breathe_fixtures_exactly() {
        let path = capture("capture-full-effects-mic-20260905.pcapng");
        let plan = extract_plan(&path, &[175, 177]).unwrap();

        assert_eq!(&plan.features()[0].bytes()[16..18], &[0xFA, 0x00]);
        assert_eq!(&plan.features()[0].bytes()[160..162], &[0xF4, 0x01]);
        assert_eq!(
            sha256(plan.features()[0].bytes()),
            "fa497c331177b29d48fd7dd701a59014ea8ad22f729a66eb362eef2e70667f39"
        );
        assert_eq!(
            sha256(plan.features()[1].bytes()),
            "955972d108e471a359dfb6da8b83613099716c4cf2edbe887a566ce396acc54b"
        );
    }

    #[test]
    fn generated_breathe_reports_match_engine_captures_byte_for_byte() {
        let connected = capture("capture-connected-modes-20260905.pcapng");
        let duration = BreatheDuration::from_seconds(10).unwrap();
        let header = Color::new(0xFF, 0x3C, 0x00);
        let effect = Color::new(0x24, 0x68, 0xAC);

        for (frame, zone, mode, reverse) in [
            (7, Zone::Right, BreatheMode::Sweep, false),
            (11, Zone::Left, BreatheMode::Sweep, false),
            (21, Zone::Right, BreatheMode::Sweep, true),
            (23, Zone::Left, BreatheMode::Sweep, true),
            (31, Zone::Right, BreatheMode::Synchronized, false),
            (33, Zone::Left, BreatheMode::Synchronized, false),
        ] {
            let captured = extract_report(&connected, frame).unwrap();
            let generated =
                FeatureReport::breathe(zone, header, effect, duration, mode, reverse).unwrap();
            assert_eq!(generated.bytes(), captured.bytes(), "frame {frame}");
        }

        let independent = capture("capture-full-effects-mic-20260905.pcapng");
        let duration = BreatheDuration::from_seconds(5).unwrap();
        let effect = Color::new(0x12, 0x34, 0x56);
        for (frame, zone, header) in [
            (175, Zone::Right, Color::new(0x01, 0x02, 0x03)),
            (177, Zone::Left, effect),
        ] {
            let captured = extract_report(&independent, frame).unwrap();
            let generated = FeatureReport::breathe(
                zone,
                header,
                effect,
                duration,
                BreatheMode::Synchronized,
                false,
            )
            .unwrap();
            assert_eq!(generated.bytes(), captured.bytes(), "frame {frame}");
        }
    }

    #[test]
    fn rejects_missing_captures_and_invalid_frames_before_tshark() {
        assert!(matches!(
            extract_plan(Path::new("does-not-exist.pcapng"), &[1]),
            Err(CaptureError::MissingCapture(_))
        ));
        let path = capture("capture-connected-modes-20260905.pcapng");
        assert!(matches!(
            extract_plan(&path, &[]),
            Err(CaptureError::InvalidFrame)
        ));
        assert!(matches!(
            extract_plan(&path, &[0]),
            Err(CaptureError::InvalidFrame)
        ));
    }
}
