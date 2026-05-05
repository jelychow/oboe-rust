use oboe_samples::sound_board::SoundBoardSynth;

fn main() {
    let mut synth = SoundBoardSynth::new(48_000, 2, 8);
    synth.note_on(3);
    println!("{:?}", synth.render(4));
}
