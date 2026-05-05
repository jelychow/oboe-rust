use oboe_samples::parselib::{write_test_wav_i16, WavData};

fn main() {
    let wav = write_test_wav_i16(1, 48_000, &[0, 16_384, -16_384]);
    let parsed = WavData::parse(&wav).unwrap();
    println!(
        "channels={} rate={} samples={:?}",
        parsed.channel_count, parsed.sample_rate, parsed.frames
    );
}
