#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AudioDevice {
    pub id: i32,
    pub name: String,
    pub is_output: bool,
    pub is_input: bool,
}

impl AudioDevice {
    pub fn new(id: i32, name: impl Into<String>, is_output: bool, is_input: bool) -> Self {
        Self {
            id,
            name: name.into(),
            is_output,
            is_input,
        }
    }
}

pub fn select_first_output(devices: &[AudioDevice]) -> Option<AudioDevice> {
    devices.iter().find(|device| device.is_output).cloned()
}

pub fn select_first_input(devices: &[AudioDevice]) -> Option<AudioDevice> {
    devices.iter().find(|device| device.is_input).cloned()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn audio_device_filters_by_direction() {
        let devices = [
            AudioDevice::new(1, "mic", false, true),
            AudioDevice::new(2, "speaker", true, false),
        ];
        assert_eq!(select_first_input(&devices).unwrap().id, 1);
        assert_eq!(select_first_output(&devices).unwrap().id, 2);
    }
}
