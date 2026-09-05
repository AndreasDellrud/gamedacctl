use std::{thread, time::Duration};

use hidapi::{HidApi, HidDevice};
use thiserror::Error;

use crate::{FeatureReport, LightingPlan, OutputReport};

pub const VENDOR_ID: u16 = 0x1038;
pub const PRODUCT_ID: u16 = 0x1280;
pub const INTERFACE_NUMBER: i32 = 0;
const REPORT_DELAY: Duration = Duration::from_millis(60);

#[derive(Debug, Error)]
pub enum TransportError {
    #[error("could not initialize HIDAPI: {0}")]
    Initialization(#[source] hidapi::HidError),
    #[error("GameDAC control interface 1038:1280 interface 0 was not found")]
    NotFound,
    #[error(
        "could not open the GameDAC control interface; check the scoped hidraw permission: {0}"
    )]
    Open(#[source] hidapi::HidError),
    #[error("GameDAC feature report failed: {0}")]
    Feature(#[source] hidapi::HidError),
    #[error("GameDAC output report failed: {0}")]
    Output(#[source] hidapi::HidError),
}

pub trait Transport {
    fn send_feature(&self, report: &FeatureReport) -> Result<(), TransportError>;
    fn send_output(&self, report: &OutputReport) -> Result<(), TransportError>;

    fn execute(&self, plan: &LightingPlan) -> Result<(), TransportError> {
        for report in plan.features() {
            self.send_feature(report)?;
            thread::sleep(REPORT_DELAY);
        }
        for report in plan.outputs() {
            self.send_output(report)?;
            thread::sleep(REPORT_DELAY);
        }
        Ok(())
    }
}

pub struct HidTransport {
    _api: HidApi,
    device: HidDevice,
    path: String,
}

impl HidTransport {
    pub fn open() -> Result<Self, TransportError> {
        let api = HidApi::new().map_err(TransportError::Initialization)?;
        let info = api
            .device_list()
            .find(|device| {
                device.vendor_id() == VENDOR_ID
                    && device.product_id() == PRODUCT_ID
                    && device.interface_number() == INTERFACE_NUMBER
            })
            .ok_or(TransportError::NotFound)?;
        let path = info.path().to_string_lossy().into_owned();
        let device = info.open_device(&api).map_err(TransportError::Open)?;
        Ok(Self {
            _api: api,
            device,
            path,
        })
    }

    pub fn path(&self) -> &str {
        &self.path
    }
}

impl Transport for HidTransport {
    fn send_feature(&self, report: &FeatureReport) -> Result<(), TransportError> {
        let mut hid_report = Vec::with_capacity(report.bytes().len() + 1);
        hid_report.push(0);
        hid_report.extend_from_slice(report.bytes());
        self.device
            .send_feature_report(&hid_report)
            .map_err(TransportError::Feature)?;
        Ok(())
    }

    fn send_output(&self, report: &OutputReport) -> Result<(), TransportError> {
        let mut hid_report = Vec::with_capacity(report.bytes().len() + 1);
        hid_report.push(0);
        hid_report.extend_from_slice(report.bytes());
        self.device
            .write(&hid_report)
            .map_err(TransportError::Output)?;
        Ok(())
    }
}
