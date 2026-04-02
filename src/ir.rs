use std::sync::{Arc, Mutex};

#[derive(Clone, Default)]
pub struct IrData {
    pub intensity: f32,
    pub depth: Option<f32>,
}

pub type SharedIr = Arc<Mutex<IrData>>;

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
    vec![]
}

pub fn create_ir_source() -> SharedIr {
    mock::create_mock_ir()
}
