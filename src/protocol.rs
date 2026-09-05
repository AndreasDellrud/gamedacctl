use std::{fmt, str::FromStr};

use thiserror::Error;

pub const FEATURE_REPORT_LEN: usize = 1024;
pub const OUTPUT_REPORT_LEN: usize = 64;

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
#[repr(u8)]
pub enum Zone {
    Left = 0,
    Right = 1,
    MicrophoneLive = 2,
    MicrophoneMuted = 3,
}

impl Zone {
    pub const fn id(self) -> u8 {
        self as u8
    }

    pub const fn mask(self) -> u8 {
        1 << self.id()
    }

    pub const fn label(self) -> &'static str {
        match self {
            Self::Left => "left",
            Self::Right => "right",
            Self::MicrophoneLive => "microphone-live",
            Self::MicrophoneMuted => "microphone-muted",
        }
    }

    fn from_id(value: u8) -> Result<Self, ProtocolError> {
        match value {
            0 => Ok(Self::Left),
            1 => Ok(Self::Right),
            2 => Ok(Self::MicrophoneLive),
            3 => Ok(Self::MicrophoneMuted),
            value => Err(ProtocolError::UnsupportedZone(value)),
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Color([u8; 3]);

impl Color {
    pub const BLACK: Self = Self([0, 0, 0]);

    pub const fn new(red: u8, green: u8, blue: u8) -> Self {
        Self([red, green, blue])
    }

    pub const fn bytes(self) -> [u8; 3] {
        self.0
    }
}

impl fmt::Display for Color {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "#{:02X}{:02X}{:02X}",
            self.0[0], self.0[1], self.0[2]
        )
    }
}

impl FromStr for Color {
    type Err = ProtocolError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        let value = value.strip_prefix('#').unwrap_or(value);
        if value.len() != 6 || !value.bytes().all(|byte| byte.is_ascii_hexdigit()) {
            return Err(ProtocolError::InvalidColor(value.to_owned()));
        }

        let red = u8::from_str_radix(&value[0..2], 16)
            .map_err(|_| ProtocolError::InvalidColor(value.to_owned()))?;
        let green = u8::from_str_radix(&value[2..4], 16)
            .map_err(|_| ProtocolError::InvalidColor(value.to_owned()))?;
        let blue = u8::from_str_radix(&value[4..6], 16)
            .map_err(|_| ProtocolError::InvalidColor(value.to_owned()))?;
        Ok(Self::new(red, green, blue))
    }
}

#[derive(Debug, Error, Eq, PartialEq)]
pub enum ProtocolError {
    #[error(
        "color must contain exactly six hexadecimal digits, optionally prefixed by #; got {0:?}"
    )]
    InvalidColor(String),
    #[error("zone {0} is not a supported GameDAC lighting zone")]
    UnsupportedZone(u8),
    #[error("feature report has {actual} bytes; expected {FEATURE_REPORT_LEN}")]
    InvalidFeatureLength { actual: usize },
    #[error("feature report does not begin with the GameDAC lighting prefix 0xAA")]
    InvalidFeaturePrefix,
    #[error("feature report zone fields disagree ({first} and {repeated})")]
    ZoneMismatch { first: u8, repeated: u8 },
    #[error("feature report uses unsupported mode marker {0:#04x}")]
    UnsupportedMode(u8),
    #[error("at least one lighting zone must be selected")]
    EmptyPlan,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FeatureReport {
    bytes: [u8; FEATURE_REPORT_LEN],
    zone: Zone,
}

impl FeatureReport {
    pub fn steady(zone: Zone, color: Color) -> Self {
        let mut bytes = [0; FEATURE_REPORT_LEN];
        let [red, green, blue] = color.bytes();
        bytes[..12].copy_from_slice(&[
            0xAA,
            zone.id(),
            red,
            green,
            blue,
            0xFF,
            0x32,
            0xC8,
            0xC8,
            0x00,
            zone.id(),
            0x01,
        ]);
        Self { bytes, zone }
    }

    pub fn captured(bytes: &[u8]) -> Result<Self, ProtocolError> {
        if bytes.len() != FEATURE_REPORT_LEN {
            return Err(ProtocolError::InvalidFeatureLength {
                actual: bytes.len(),
            });
        }
        if bytes[0] != 0xAA {
            return Err(ProtocolError::InvalidFeaturePrefix);
        }
        if bytes[1] != bytes[10] {
            return Err(ProtocolError::ZoneMismatch {
                first: bytes[1],
                repeated: bytes[10],
            });
        }
        let zone = Zone::from_id(bytes[1])?;
        if !matches!(bytes[11], 0 | 1) {
            return Err(ProtocolError::UnsupportedMode(bytes[11]));
        }

        let mut report = [0; FEATURE_REPORT_LEN];
        report.copy_from_slice(bytes);
        Ok(Self {
            bytes: report,
            zone,
        })
    }

    pub const fn zone(&self) -> Zone {
        self.zone
    }

    pub const fn bytes(&self) -> &[u8; FEATURE_REPORT_LEN] {
        &self.bytes
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct OutputReport([u8; OUTPUT_REPORT_LEN]);

impl OutputReport {
    fn from_prefix(prefix: &[u8]) -> Self {
        let mut bytes = [0; OUTPUT_REPORT_LEN];
        bytes[..prefix.len()].copy_from_slice(prefix);
        Self(bytes)
    }

    pub fn apply(mask: u8) -> Self {
        Self::from_prefix(&[0xA5, mask, 0x0A])
    }

    pub fn save() -> Self {
        Self::from_prefix(&[0xAC])
    }

    pub fn finish() -> Self {
        Self::from_prefix(&[0x09])
    }

    pub const fn bytes(&self) -> &[u8; OUTPUT_REPORT_LEN] {
        &self.0
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LightingPlan {
    features: Vec<FeatureReport>,
    outputs: Vec<OutputReport>,
}

impl LightingPlan {
    pub fn steady(
        settings: impl IntoIterator<Item = (Zone, Color)>,
    ) -> Result<Self, ProtocolError> {
        let features: Vec<_> = settings
            .into_iter()
            .map(|(zone, color)| FeatureReport::steady(zone, color))
            .collect();
        let mask = zone_mask(&features)?;
        Ok(Self {
            features,
            outputs: vec![
                OutputReport::apply(mask),
                OutputReport::save(),
                OutputReport::finish(),
            ],
        })
    }

    pub fn captured(features: Vec<FeatureReport>) -> Result<Self, ProtocolError> {
        let mask = zone_mask(&features)?;
        Ok(Self {
            features,
            outputs: vec![
                OutputReport::apply(mask),
                OutputReport::save(),
                OutputReport::finish(),
            ],
        })
    }

    pub fn features(&self) -> &[FeatureReport] {
        &self.features
    }

    pub fn outputs(&self) -> &[OutputReport] {
        &self.outputs
    }

    pub fn zone_mask(&self) -> u8 {
        self.features
            .iter()
            .fold(0, |mask, report| mask | report.zone().mask())
    }
}

fn zone_mask(features: &[FeatureReport]) -> Result<u8, ProtocolError> {
    if features.is_empty() {
        return Err(ProtocolError::EmptyPlan);
    }
    Ok(features
        .iter()
        .fold(0, |mask, report| mask | report.zone().mask()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_and_displays_colors() {
        assert_eq!(
            "#123456".parse::<Color>().unwrap().bytes(),
            [0x12, 0x34, 0x56]
        );
        assert_eq!("aBcDeF".parse::<Color>().unwrap().to_string(), "#ABCDEF");
        for value in ["", "12345", "1234567", "GG0000", "##123456"] {
            assert!(matches!(
                value.parse::<Color>(),
                Err(ProtocolError::InvalidColor(_))
            ));
        }
    }

    #[test]
    fn steady_report_matches_verified_layout_byte_for_byte() {
        for (zone, color) in [
            (Zone::Left, Color::new(0xFF, 0x00, 0x00)),
            (Zone::Right, Color::new(0x00, 0x00, 0xFF)),
            (Zone::MicrophoneLive, Color::new(0x00, 0xFF, 0x00)),
            (Zone::MicrophoneMuted, Color::new(0xFF, 0x00, 0x00)),
        ] {
            let report = FeatureReport::steady(zone, color);
            assert_eq!(
                &report.bytes()[..12],
                &[
                    0xAA,
                    zone.id(),
                    color.bytes()[0],
                    color.bytes()[1],
                    color.bytes()[2],
                    0xFF,
                    0x32,
                    0xC8,
                    0xC8,
                    0x00,
                    zone.id(),
                    0x01
                ]
            );
            assert!(report.bytes()[12..].iter().all(|byte| *byte == 0));
        }
    }

    #[test]
    fn plan_computes_exact_zone_mask_and_output_reports() {
        let plan = LightingPlan::steady([
            (Zone::Left, Color::new(1, 2, 3)),
            (Zone::MicrophoneMuted, Color::new(4, 5, 6)),
        ])
        .unwrap();
        assert_eq!(plan.zone_mask(), 0x09);
        assert_eq!(&plan.outputs()[0].bytes()[..3], &[0xA5, 0x09, 0x0A]);
        assert!(plan.outputs()[0].bytes()[3..].iter().all(|byte| *byte == 0));
        assert_eq!(plan.outputs()[1].bytes()[0], 0xAC);
        assert_eq!(plan.outputs()[2].bytes()[0], 0x09);
    }

    #[test]
    fn captured_reports_fail_closed() {
        assert_eq!(
            FeatureReport::captured(&[0; 12]).unwrap_err(),
            ProtocolError::InvalidFeatureLength { actual: 12 }
        );

        let mut report = *FeatureReport::steady(Zone::Left, Color::BLACK).bytes();
        report[0] = 0;
        assert_eq!(
            FeatureReport::captured(&report).unwrap_err(),
            ProtocolError::InvalidFeaturePrefix
        );

        let mut report = *FeatureReport::steady(Zone::Left, Color::BLACK).bytes();
        report[10] = 1;
        assert!(matches!(
            FeatureReport::captured(&report),
            Err(ProtocolError::ZoneMismatch { .. })
        ));

        let mut report = *FeatureReport::steady(Zone::Left, Color::BLACK).bytes();
        report[1] = 4;
        report[10] = 4;
        assert_eq!(
            FeatureReport::captured(&report).unwrap_err(),
            ProtocolError::UnsupportedZone(4)
        );

        let mut report = *FeatureReport::steady(Zone::Left, Color::BLACK).bytes();
        report[11] = 2;
        assert_eq!(
            FeatureReport::captured(&report).unwrap_err(),
            ProtocolError::UnsupportedMode(2)
        );
    }

    #[test]
    fn empty_plans_are_rejected() {
        assert_eq!(
            LightingPlan::steady([]).unwrap_err(),
            ProtocolError::EmptyPlan
        );
        assert_eq!(
            LightingPlan::captured(Vec::new()).unwrap_err(),
            ProtocolError::EmptyPlan
        );
    }
}
