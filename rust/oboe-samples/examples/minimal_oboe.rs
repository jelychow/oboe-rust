use oboe_samples::minimal_oboe::SimpleNoiseMaker;

fn main() {
    let mut noise = SimpleNoiseMaker::new(2, 0x600d);
    println!("{:?}", noise.render(4));
}
