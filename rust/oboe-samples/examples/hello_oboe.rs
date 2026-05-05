use oboe_samples::hello_oboe::HelloOboeSample;

fn main() {
    let mut sample = HelloOboeSample::new(48_000, 2);
    sample.tap(true);
    println!("{:?}", sample.render(4));
}
