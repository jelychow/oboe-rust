use oboe_samples::audio_device::{select_first_output, AudioDevice};

fn main() {
    let devices = [
        AudioDevice::new(1, "built-in mic", false, true),
        AudioDevice::new(2, "speaker", true, false),
    ];
    println!("{:?}", select_first_output(&devices));
}
