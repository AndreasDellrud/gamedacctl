//! Safe packet construction and transport for original GameDAC lighting.

pub mod capture;
pub mod protocol;
pub mod transport;

pub use protocol::{
    BreatheDuration, BreatheMode, Color, FeatureReport, LightingPlan, OutputReport, ProtocolError,
    Zone,
};
