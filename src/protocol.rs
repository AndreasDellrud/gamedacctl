use std::{fmt, str::FromStr};

use serde::{Deserialize, Deserializer, Serialize, Serializer, de::Error as _};
use thiserror::Error;

pub const FEATURE_REPORT_LEN: usize = 1024;
pub const OUTPUT_REPORT_LEN: usize = 64;
pub const MAX_COLOR_SHIFT_COLORS: usize = 14;
pub const MAX_MULTI_COLOR_BREATHE_COLORS: usize = 4;

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
#[repr(u8)]
pub enum Zone {
    Left = 0,
    Right = 1,
    MicrophoneLive = 2,
    MicrophoneMuted = 3,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BreatheMode {
    Synchronized,
    Sweep,
}

impl BreatheMode {
    const fn phase_flag(self) -> u8 {
        match self {
            Self::Synchronized => 0,
            Self::Sweep => 1,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct BreatheDuration {
    seconds: u16,
}

impl BreatheDuration {
    pub fn from_seconds(seconds: u16) -> Result<Self, ProtocolError> {
        if !(1..=30).contains(&seconds) {
            return Err(ProtocolError::InvalidBreatheDuration(seconds));
        }
        Ok(Self { seconds })
    }

    pub const fn seconds(self) -> u16 {
        self.seconds
    }

    const fn centiseconds(self) -> u16 {
        self.seconds * 100
    }
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

impl Serialize for Color {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

impl<'de> Deserialize<'de> for Color {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        String::deserialize(deserializer)?
            .parse()
            .map_err(D::Error::custom)
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
    #[error("Breathe duration must be a whole number from 1 through 30 seconds; got {0}")]
    InvalidBreatheDuration(u16),
    #[error("reverse direction is observed only for connected Sweep mode")]
    ReverseRequiresSweep,
    #[error("connected Sweep is verified only for a single Breathe color; got {0}")]
    SweepRequiresSingleColor(usize),
    #[error("{effect} requires between {min} and {max} colors; got {actual}")]
    InvalidEffectColorCount {
        effect: &'static str,
        min: usize,
        max: usize,
        actual: usize,
    },
    #[error(
        "{effect} transition {transition} is too fast for the captured signed-byte coefficient layout; increase the duration"
    )]
    TransitionRateOutOfRange {
        effect: &'static str,
        transition: usize,
    },
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

    pub fn breathe(
        zone: Zone,
        header_color: Color,
        effect_color: Color,
        duration: BreatheDuration,
        mode: BreatheMode,
        reverse: bool,
    ) -> Result<Self, ProtocolError> {
        if reverse && mode != BreatheMode::Sweep {
            return Err(ProtocolError::ReverseRequiresSweep);
        }
        Self::multi_color_breathe(zone, header_color, &[effect_color], duration, mode, reverse)
    }

    pub fn color_shift(
        zone: Zone,
        header_color: Color,
        colors: &[Color],
        duration: BreatheDuration,
    ) -> Result<Self, ProtocolError> {
        validate_color_count("ColorShift", colors, 2, MAX_COLOR_SHIFT_COLORS)?;
        let targets = colors[1..]
            .iter()
            .copied()
            .chain(std::iter::once(colors[0]))
            .collect::<Vec<_>>();
        Self::animation(
            zone,
            header_color,
            colors[0],
            &targets,
            duration,
            BreatheMode::Synchronized,
            false,
            "ColorShift",
        )
    }

    pub fn multi_color_breathe(
        zone: Zone,
        header_color: Color,
        colors: &[Color],
        duration: BreatheDuration,
        mode: BreatheMode,
        reverse: bool,
    ) -> Result<Self, ProtocolError> {
        if reverse && mode != BreatheMode::Sweep {
            return Err(ProtocolError::ReverseRequiresSweep);
        }
        validate_color_count(
            "Multi Color Breathe",
            colors,
            1,
            MAX_MULTI_COLOR_BREATHE_COLORS,
        )?;
        if mode == BreatheMode::Sweep && colors.len() != 1 {
            return Err(ProtocolError::SweepRequiresSingleColor(colors.len()));
        }

        let mut targets = Vec::with_capacity(colors.len() * 2);
        for color in colors.iter().skip(1) {
            targets.push(Color::BLACK);
            targets.push(*color);
        }
        targets.push(Color::BLACK);
        targets.push(colors[0]);
        Self::animation(
            zone,
            header_color,
            colors[0],
            &targets,
            duration,
            mode,
            reverse,
            "Multi Color Breathe",
        )
    }

    #[allow(clippy::too_many_arguments)]
    fn animation(
        zone: Zone,
        header_color: Color,
        initial_color: Color,
        targets: &[Color],
        duration: BreatheDuration,
        mode: BreatheMode,
        reverse: bool,
        effect: &'static str,
    ) -> Result<Self, ProtocolError> {
        let mut bytes = [0; FEATURE_REPORT_LEN];
        let [header_red, header_green, header_blue] = header_color.bytes();
        bytes[..12].copy_from_slice(&[
            0xAA,
            zone.id(),
            header_red,
            header_green,
            header_blue,
            0xFF,
            0x32,
            0xC8,
            0xC8,
            0x00,
            zone.id(),
            0x00,
        ]);

        let ticks = distribute_ticks(duration.centiseconds(), targets.len());
        let mut from = initial_color;
        for (index, (target, transition_ticks)) in targets.iter().copied().zip(ticks).enumerate() {
            let offset = 12 + index * 8;
            for (channel_offset, (from_channel, to_channel)) in
                from.bytes().into_iter().zip(target.bytes()).enumerate()
            {
                let delta = ((i32::from(to_channel) - i32::from(from_channel)) << 4)
                    / i32::from(transition_ticks);
                let rate =
                    i8::try_from(delta).map_err(|_| ProtocolError::TransitionRateOutOfRange {
                        effect,
                        transition: index + 1,
                    })?;
                bytes[offset + channel_offset] = rate as u8;
            }
            bytes[offset + 4..offset + 6].copy_from_slice(&transition_ticks.to_le_bytes());
            bytes[offset + 6] = if index + 1 == targets.len() {
                0
            } else {
                (index + 1) as u8
            };
            from = target;
        }

        for (channel, offset) in initial_color.bytes().into_iter().zip([140, 142, 144]) {
            bytes[offset..offset + 2].copy_from_slice(&((channel as u16) << 4).to_le_bytes());
        }
        bytes[146] = 0xFF;
        bytes[152] = mode.phase_flag();
        bytes[156] = 0x01;
        bytes[158] = targets.len() as u8;
        bytes[160..162].copy_from_slice(&duration.centiseconds().to_le_bytes());
        bytes[162] = u8::from(reverse);

        Ok(Self { bytes, zone })
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

    pub fn with_steady(
        mut self,
        settings: impl IntoIterator<Item = (Zone, Color)>,
    ) -> Result<Self, ProtocolError> {
        self.features.extend(
            settings
                .into_iter()
                .map(|(zone, color)| FeatureReport::steady(zone, color)),
        );
        let mask = zone_mask(&self.features)?;
        self.outputs = vec![
            OutputReport::apply(mask),
            OutputReport::save(),
            OutputReport::finish(),
        ];
        Ok(self)
    }

    pub fn breathe(
        color: Color,
        duration: BreatheDuration,
        mode: BreatheMode,
        reverse: bool,
    ) -> Result<Self, ProtocolError> {
        let features = [Zone::Right, Zone::Left]
            .into_iter()
            .map(|zone| FeatureReport::breathe(zone, color, color, duration, mode, reverse))
            .collect::<Result<Vec<_>, _>>()?;
        Self::captured(features)
    }

    pub fn color_shift(colors: &[Color], duration: BreatheDuration) -> Result<Self, ProtocolError> {
        validate_color_count("ColorShift", colors, 2, 2)?;
        let features = [Zone::Right, Zone::Left]
            .into_iter()
            .map(|zone| FeatureReport::color_shift(zone, colors[0], colors, duration))
            .collect::<Result<Vec<_>, _>>()?;
        Self::captured(features)
    }

    pub fn multi_color_breathe(
        colors: &[Color],
        duration: BreatheDuration,
    ) -> Result<Self, ProtocolError> {
        validate_color_count(
            "Multi Color Breathe",
            colors,
            1,
            MAX_MULTI_COLOR_BREATHE_COLORS,
        )?;
        let features = [Zone::Right, Zone::Left]
            .into_iter()
            .map(|zone| {
                FeatureReport::multi_color_breathe(
                    zone,
                    colors[0],
                    colors,
                    duration,
                    BreatheMode::Synchronized,
                    false,
                )
            })
            .collect::<Result<Vec<_>, _>>()?;
        Self::captured(features)
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

fn validate_color_count(
    effect: &'static str,
    colors: &[Color],
    min: usize,
    max: usize,
) -> Result<(), ProtocolError> {
    if !(min..=max).contains(&colors.len()) {
        return Err(ProtocolError::InvalidEffectColorCount {
            effect,
            min,
            max,
            actual: colors.len(),
        });
    }
    Ok(())
}

fn distribute_ticks(total: u16, count: usize) -> Vec<u16> {
    let count_u16 = count as u16;
    let average = total.div_ceil(count_u16);
    let rounded = average.div_ceil(10) * 10;
    if rounded * (count_u16 - 1) < total {
        let mut ticks = vec![rounded; count - 1];
        ticks.push(total - rounded * (count_u16 - 1));
        ticks
    } else {
        let base = total / count_u16;
        let remainder = total % count_u16;
        (0..count)
            .map(|index| base + u16::from((index as u16) < remainder))
            .collect()
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

    #[test]
    fn breathe_duration_is_restricted_to_captured_safe_range() {
        assert_eq!(
            BreatheDuration::from_seconds(0).unwrap_err(),
            ProtocolError::InvalidBreatheDuration(0)
        );
        assert_eq!(BreatheDuration::from_seconds(1).unwrap().seconds(), 1);
        assert_eq!(BreatheDuration::from_seconds(30).unwrap().seconds(), 30);
        assert_eq!(
            BreatheDuration::from_seconds(31).unwrap_err(),
            ProtocolError::InvalidBreatheDuration(31)
        );
    }

    #[test]
    fn color_sequence_counts_and_transition_shapes_are_bounded() {
        let duration = BreatheDuration::from_seconds(10).unwrap();
        let rainbow = [
            Color::new(0xFF, 0x00, 0x00),
            Color::new(0xFF, 0xFF, 0x00),
            Color::new(0x00, 0xFF, 0x00),
            Color::new(0x00, 0xFF, 0xFF),
            Color::new(0x00, 0x00, 0xFF),
            Color::new(0xFF, 0x00, 0xFF),
        ];
        let shift = FeatureReport::color_shift(
            Zone::MicrophoneLive,
            Color::new(4, 5, 6),
            &rainbow,
            duration,
        )
        .unwrap();
        assert_eq!(shift.bytes()[158], 6);
        assert_eq!(&shift.bytes()[160..162], &[0xE8, 0x03]);
        assert_eq!(&shift.bytes()[12..20], &[0, 24, 0, 0, 170, 0, 1, 0]);
        assert_eq!(&shift.bytes()[52..60], &[0, 0, 0xE5, 0, 150, 0, 0, 0]);

        let breathe = FeatureReport::multi_color_breathe(
            Zone::Left,
            rainbow[0],
            &rainbow[..3],
            duration,
            BreatheMode::Synchronized,
            false,
        )
        .unwrap();
        assert_eq!(breathe.bytes()[158], 6);
        assert_eq!(&breathe.bytes()[140..146], &[0xF0, 0x0F, 0, 0, 0, 0]);

        assert!(matches!(
            FeatureReport::color_shift(Zone::Left, Color::BLACK, &rainbow[..1], duration),
            Err(ProtocolError::InvalidEffectColorCount { .. })
        ));
        assert_eq!(
            FeatureReport::multi_color_breathe(
                Zone::Left,
                rainbow[0],
                &rainbow[..2],
                duration,
                BreatheMode::Sweep,
                false,
            )
            .unwrap_err(),
            ProtocolError::SweepRequiresSingleColor(2)
        );
        assert!(matches!(
            FeatureReport::multi_color_breathe(
                Zone::Left,
                Color::BLACK,
                &rainbow[..5],
                duration,
                BreatheMode::Synchronized,
                false,
            ),
            Err(ProtocolError::InvalidEffectColorCount { .. })
        ));
    }

    #[test]
    fn reverse_is_rejected_outside_observed_sweep_mode() {
        assert_eq!(
            FeatureReport::breathe(
                Zone::Left,
                Color::BLACK,
                Color::new(0x12, 0x34, 0x56),
                BreatheDuration::from_seconds(5).unwrap(),
                BreatheMode::Synchronized,
                true,
            )
            .unwrap_err(),
            ProtocolError::ReverseRequiresSweep
        );
    }
}
