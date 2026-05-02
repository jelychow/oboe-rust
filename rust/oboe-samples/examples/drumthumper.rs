use oboe_samples::drumthumper::DrumThumper;

fn main() {
    let mut drums = DrumThumper::new(2);
    drums.load_pad(0, vec![1.0, 0.5], 1, -1.0).unwrap();
    drums.trigger(0).unwrap();
    println!("{:?}", drums.render(2));
}
