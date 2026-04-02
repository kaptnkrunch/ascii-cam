use std::sync::{Arc, Mutex};

#[derive(Clone, Default)]
pub struct IrData {
    pub intensity: f32,
    pub depth: Option<f32>,
    #[allow(dead_code)]
    pub left_ir: Vec<u8>,
    #[allow(dead_code)]
    pub right_ir: Vec<u8>,
}

pub type SharedIr = Arc<Mutex<IrData>>;

#[cfg(feature = "realsense")]
pub mod realsense {
    use super::*;
    use realsense_sdk::context::Context;
    use realsense_sdk::device::Device;
    use realsense_sdk::frame::{Frame, FrameType};

    pub struct RealSenseDevice {
        device: Device,
        shared: SharedIr,
    }

    impl RealSenseDevice {
        pub fn new() -> Option<Self> {
            let ctx = Context::new()?;
            let device = ctx.query_devices().into_iter().next()?;
            Some(Self {
                device,
                shared: Arc::new(Mutex::new(IrData::default())),
            })
        }

        pub fn start(&self) -> anyhow::Result<()> {
            let shared = self.shared.clone();
            self.device.start(FrameType::STEREO)?;

            std::thread::spawn(move || {
                while let Ok(frame) = self.device.poll_frame() {
                    if let Some(ir_frame) = frame.ir_left() {
                        let mut data = shared.lock().unwrap();
                        data.left_ir = ir_frame.to_vec();
                        data.intensity = calculate_intensity(&data.left_ir);
                    }
                    if let Some(depth_frame) = frame.depth() {
                        let mut data = shared.lock().unwrap();
                        data.depth = Some(calculate_depth_avg(&depth_frame));
                    }
                }
            });

            Ok(())
        }

        pub fn get_data(&self) -> SharedIr {
            self.shared.clone()
        }
    }

    fn calculate_intensity(ir_data: &[u8]) -> f32 {
        if ir_data.is_empty() {
            return 0.0;
        }
        let sum: u32 = ir_data.iter().map(|&p| p as u32).sum();
        (sum as f32 / ir_data.len() as f32 / 255.0).min(1.0)
    }

    fn calculate_depth_avg(depth: &[u16]) -> f32 {
        if depth.is_empty() {
            return 0.0;
        }
        let sum: u32 = depth.iter().map(|&d| d as u32).sum();
        let avg = sum as f32 / depth.len() as f32;
        (avg / 65535.0).min(1.0)
    }
}

pub mod mock {
    use super::*;

    pub fn create_mock_ir() -> SharedIr {
        let shared: SharedIr = Arc::new(Mutex::new(IrData::default()));

        let shared_clone = shared.clone();
        std::thread::spawn(move || loop {
            {
                let mut data = shared_clone.lock().unwrap();
                data.intensity = (data.intensity + 0.01) % 1.0;
                data.depth = Some(data.intensity * 0.8);
            }
            std::thread::sleep(std::time::Duration::from_millis(33));
        });

        shared
    }
}

pub fn list_depth_cameras() -> Vec<String> {
    #[cfg(feature = "realsense")]
    {
        realsense::RealSenseDevice::new()
            .map(|_| vec!["RealSense Camera".to_string()])
            .unwrap_or_default()
    }
    #[cfg(not(feature = "realsense"))]
    {
        vec![]
    }
}

pub fn create_ir_source() -> SharedIr {
    #[cfg(feature = "realsense")]
    {
        if let Some(device) = realsense::RealSenseDevice::new() {
            if device.start().is_ok() {
                return device.get_data();
            }
        }
    }
    mock::create_mock_ir()
}
