//! Safe packet construction and transport for original GameDAC lighting.

pub mod capture;
pub mod profile;
pub mod protocol;
pub mod transport;

pub use protocol::{
    BreatheDuration, BreatheMode, Color, ColorMarker, ColorShiftDuration, ColorShiftMode,
    FeatureReport, LightingPlan, OutputReport, ProtocolError, Zone,
};
