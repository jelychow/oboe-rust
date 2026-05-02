use oboe_samples::mega_drone::MegaDroneSynth;

fn main() {
    let mut synth = MegaDroneSynth::new(48_000, 2);
    synth.tap(true);
    println!("{:?}", synth.render(2));
}
