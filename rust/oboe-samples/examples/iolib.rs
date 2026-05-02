use oboe_samples::iolib::{OneShotSampleSource, SampleBuffer};

fn main() {
    let buffer = SampleBuffer::new(vec![1.0, 0.5], 1, 48_000).unwrap();
    let mut source = OneShotSampleSource::new(buffer, 0.0);
    source.trigger();
    let mut output = vec![0.0; 4];
    source.mix_into(&mut output, 2, 2);
    println!("{:?}", output);
}
