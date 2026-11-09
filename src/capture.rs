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
    if !compact.len().is_multiple_of(2) || !compact.bytes().all(|byte| byte.is_ascii_hexdigit()) {
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
        Sha256::digest(bytes)
            .iter()
            .map(|byte| format!("{byte:02x}"))
            .collect()
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
    fn generated_color_shift_matches_two_color_fixtures_and_six_record_shape() {
        let microphone = capture("capture-full-effects-mic-20260905.pcapng");
        let first = Color::new(0x12, 0x34, 0x56);
        let second = Color::new(0xA1, 0xB2, 0xC3);
        let duration = BreatheDuration::from_seconds(15).unwrap();
        for (frame, colors) in [(361, [first, second]), (377, [second, first])] {
            let captured = extract_report(&microphone, frame).unwrap();
            let generated =
                FeatureReport::color_shift(Zone::MicrophoneLive, first, &colors, duration).unwrap();
            assert_eq!(generated.bytes(), captured.bytes(), "frame {frame}");
        }

        // The longer GG palette was not fully recorded. This primary-color loop
        // reconstructs its packet bytes, but is structural evidence only; the
        // product boundary remains the correlated two-color case above.
        let presets = capture("capture-effect-presets-20260905.pcapng");
        let rainbow = [
            Color::new(0xFF, 0x00, 0x00),
            Color::new(0xFF, 0xFF, 0x00),
            Color::new(0x00, 0xFF, 0x00),
            Color::new(0x00, 0xFF, 0xFF),
            Color::new(0x00, 0x00, 0xFF),
            Color::new(0xFF, 0x00, 0xFF),
        ];
        let captured = extract_report(&presets, 37).unwrap();
        let generated = FeatureReport::color_shift(
            Zone::MicrophoneLive,
            Color::new(0x04, 0x05, 0x06),
            &rainbow,
            BreatheDuration::from_seconds(10).unwrap(),
        )
        .unwrap();
        assert_eq!(generated.bytes(), captured.bytes(), "frame 37");
    }

    #[test]
    fn reconstructs_gg122_marker_timing_and_reflected_palettes_byte_for_byte() {
        let path = capture("capture-reflected-timing-controlled-20261021.pcapng");
        assert_eq!(
            sha256(&std::fs::read(&path).unwrap()),
            "1feb516c59e9e27789e89bb36a0231422536c7d4ff56c4dc1a35f878d273398f"
        );

        // Independent reconstruction plus production comparison against immutable
        // captures. Generated native reports still need physical acceptance.
        // Reflected mirrors one palette instead of setting the wire reverse bit.
        for (frame, zone, reflected, centiseconds) in [
            (1, Zone::Right, false, 1200_u16),
            (3, Zone::Left, false, 1200),
            (11, Zone::Right, true, 1200),
            (19, Zone::Right, false, 1200),
            (21, Zone::Left, true, 1200),
            (29, Zone::Left, false, 1200),
            (37, Zone::Right, false, 1234),
            (39, Zone::Left, false, 1234),
        ] {
            let red = Color::new(255, 0, 0);
            let green = Color::new(0, 255, 0);
            let blue = Color::new(0, 0, 255);
            let palette: [(Color, u16); 3] = if reflected {
                [(red, 0), (blue, 30), (green, 75)]
            } else {
                [(red, 0), (green, 25), (blue, 70)]
            };

            let mut expected = [0_u8; 1024];
            expected[..12].copy_from_slice(&[
                0xAA,
                zone.id(),
                0xFF,
                0x3C,
                0,
                0xFF,
                0x32,
                0xC8,
                0xC8,
                0,
                zone.id(),
                0,
            ]);
            let mut aggregate = 0_u16;
            for index in 0..palette.len() {
                let (from, position) = palette[index];
                let to = palette[(index + 1) % palette.len()].0;
                let next_position = palette.get(index + 1).map_or(100, |(_, p)| *p);
                let ticks =
                    (u32::from(centiseconds) * u32::from(next_position - position) / 100) as u16;
                aggregate += ticks;
                let offset = 12 + index * 8;
                for (channel, (start, end)) in from.bytes().into_iter().zip(to.bytes()).enumerate()
                {
                    let rate = ((i32::from(end) - i32::from(start)) << 4) / i32::from(ticks);
                    expected[offset + channel] = i8::try_from(rate).unwrap() as u8;
                }
                expected[offset + 4..offset + 6].copy_from_slice(&ticks.to_le_bytes());
                expected[offset + 6] = if index == 2 { 0 } else { (index + 1) as u8 };
            }
            expected[140..146].copy_from_slice(&[0xF0, 0x0F, 0, 0, 0, 0]);
            expected[146] = 0xFF;
            expected[156] = 1;
            expected[158] = 3;
            expected[160..162].copy_from_slice(&aggregate.to_le_bytes());

            let captured = extract_report(&path, frame).unwrap();
            assert_eq!(expected.as_slice(), captured.bytes(), "frame {frame}");
            let markers = [
                crate::ColorMarker {
                    color: red,
                    percent: 0,
                },
                crate::ColorMarker {
                    color: green,
                    percent: 25,
                },
                crate::ColorMarker {
                    color: blue,
                    percent: 70,
                },
            ];
            let generated = FeatureReport::color_shift_markers(
                zone,
                Color::new(255, 60, 0),
                &markers,
                crate::ColorShiftDuration::from_centiseconds(centiseconds).unwrap(),
                reflected,
            )
            .unwrap();
            assert_eq!(
                generated.bytes(),
                captured.bytes(),
                "production frame {frame}"
            );
            let gg122 = FeatureReport::color_shift_markers_gg122(
                zone,
                Color::new(255, 60, 0),
                &markers,
                crate::ColorShiftDuration::from_centiseconds(centiseconds).unwrap(),
                reflected,
            )
            .unwrap();
            assert_eq!(gg122.bytes(), captured.bytes(), "GG timing frame {frame}");
            assert_eq!(aggregate, if centiseconds == 1234 { 1233 } else { 1200 });
        }
    }

    #[test]
    fn production_marker_plans_match_controlled_earcup_pairs() {
        use crate::{ColorMarker, ColorShiftDuration, ColorShiftMode};
        let path = capture("capture-reflected-timing-controlled-20261021.pcapng");
        let markers =
            ["FF0000@0", "00FF00@25", "0000FF@70"].map(|v| v.parse::<ColorMarker>().unwrap());
        for (frames, mode, reverse, ticks) in [
            ([1, 3], ColorShiftMode::Synchronized, false, 1200),
            ([11, 3], ColorShiftMode::Reflected, false, 1200),
            ([19, 21], ColorShiftMode::Reflected, true, 1200),
            ([37, 39], ColorShiftMode::Synchronized, false, 1234),
        ] {
            let plan = LightingPlan::color_shift_markers(
                &markers,
                ColorShiftDuration::from_centiseconds(ticks).unwrap(),
                mode,
                reverse,
            )
            .unwrap();
            for (report, frame) in plan.features().iter().zip(frames) {
                let captured = extract_report(&path, frame).unwrap();
                let mut expected = *captured.bytes();
                // Product generation sets the retained header to the first color.
                expected[2..5].copy_from_slice(&markers[0].color.bytes());
                assert_eq!(report.bytes(), &expected, "plan frame {frame}");
            }
            assert_eq!(plan.outputs()[0].bytes()[..3], [0xA5, 3, 0x0A]);
        }
    }

    #[test]
    fn recovered_breathe_integer_span_model_matches_old_fractional_preset() {
        // Independent research reconstruction from the GG122 descriptor findings.
        // No proprietary resource or interpreter is required by this test.
        // Breathe halves the integer percentage span BEFORE scaling by speed.
        let colors = [
            Color::new(255, 0, 0),
            Color::new(255, 115, 0),
            Color::new(255, 157, 0),
        ];
        let positions = [0_u16, 33, 66, 100];
        let mut expected = [0_u8; 1024];
        expected[..12].copy_from_slice(&[0xAA, 2, 1, 2, 3, 0xFF, 0x32, 0xC8, 0xC8, 0, 2, 0]);
        let mut aggregate = 0_u16;
        let mut initial = colors[0];
        for index in 0..colors.len() {
            let half_span = (positions[index + 1] - positions[index]) / 2;
            let ticks = (1350 * u32::from(half_span) / 100).max(33) as u16;
            for (part, target) in [Color::BLACK, colors[(index + 1) % colors.len()]]
                .into_iter()
                .enumerate()
            {
                let record = index * 2 + part;
                let offset = 12 + record * 8;
                for (channel, (from, to)) in
                    initial.bytes().into_iter().zip(target.bytes()).enumerate()
                {
                    let rate = (16 * (i32::from(to) - i32::from(from))) / i32::from(ticks);
                    expected[offset + channel] = i8::try_from(rate).unwrap() as u8;
                }
                expected[offset + 4..offset + 6].copy_from_slice(&ticks.to_le_bytes());
                expected[offset + 6] = if record == 5 { 0 } else { (record + 1) as u8 };
                aggregate += ticks;
                initial = target;
            }
        }
        expected[140..146].copy_from_slice(&[0xF0, 0x0F, 0, 0, 0, 0]);
        expected[146] = 255;
        expected[156] = 1;
        expected[158] = 6;
        expected[160..162].copy_from_slice(&aggregate.to_le_bytes());
        assert_eq!(aggregate, 1322);
        let captured =
            extract_report(&capture("capture-effect-presets-20260905.pcapng"), 21).unwrap();
        assert_eq!(expected.as_slice(), captured.bytes());
        let markers = colors
            .into_iter()
            .zip([0, 33, 66])
            .map(|(color, percent)| crate::ColorMarker { color, percent })
            .collect::<Vec<_>>();
        let generated = FeatureReport::breathe_markers(
            Zone::MicrophoneLive,
            Color::new(1, 2, 3),
            &markers,
            crate::ColorShiftDuration::from_centiseconds(1350).unwrap(),
        )
        .unwrap();
        assert_eq!(
            generated.bytes(),
            captured.bytes(),
            "production positioned Breathe"
        );
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
