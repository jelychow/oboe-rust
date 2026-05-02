use oboe_samples::live_effect::process_full_duplex;

fn main() {
    let input = [1.0, -1.0, 0.5, -0.5];
    println!("{:?}", process_full_duplex(&input, 2, 3));
}
