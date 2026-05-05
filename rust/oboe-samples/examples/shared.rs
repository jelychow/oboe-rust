use oboe_samples::shared::{mono_to_stereo, Oscillator};

fn main() {
    let mut oscillator = Oscillator::new(48_000, 440.0, 0.5);
    oscillator.set_wave_on(true);
    let mut mono = [0.0; 4];
    oscillator.render_mono(&mut mono);
    println!("{:?}", mono_to_stereo(&mono));
}
