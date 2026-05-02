use oboe_samples::{
    audio_device, debug_utils, drumthumper, hello_oboe, iolib, live_effect, mega_drone,
    minimal_oboe, parselib, powerplay, rhythm_game, shared, sound_board,
};

#[test]
fn shared_audio_primitives_cover_upstream_helpers() {
    let mut oscillator = shared::Oscillator::new(48_000, 440.0, 0.5);
    oscillator.set_wave_on(true);
    let mut frames = [0.0; 4];
    oscillator.render_mono(&mut frames);
    assert_eq!(frames[0], -0.5);

    let stereo = shared::mono_to_stereo(&[0.25, -0.5]);
    assert_eq!(stereo, vec![0.25, 0.25, -0.5, -0.5]);

    let mut trace = debug_utils::Trace::enabled();
    trace.begin_section("render");
    assert_eq!(
        trace.active_sections().first().map(String::as_str),
        Some("render")
    );
    trace.end_section();
    assert!(trace.active_sections().is_empty());
}

#[test]
fn app_samples_render_or_transform_audio_without_android_runtime() {
    let mut hello = hello_oboe::HelloOboeSample::new(48_000, 2);
    hello.tap(true);
    assert!(hello.render(8).iter().any(|sample| *sample != 0.0));

    let mut noise = minimal_oboe::SimpleNoiseMaker::new(2, 0x1234);
    assert_eq!(noise.render(4).len(), 8);

    let output = live_effect::process_full_duplex(&[1.0, -1.0], 2, 3);
    assert_eq!(output, vec![0.95, -0.95, 0.0, 0.0, 0.0, 0.0]);

    let mut drone = mega_drone::MegaDroneSynth::new(48_000, 2);
    drone.tap(true);
    assert_eq!(drone.render(4).len(), 8);

    let mut board = sound_board::SoundBoardSynth::new(48_000, 2, 3);
    board.note_on(1);
    assert_eq!(board.render(4).len(), 8);
}

#[test]
fn library_samples_parse_and_mix_assets() {
    let wav = parselib::write_test_wav_i16(1, 48_000, &[0, 16_384, -16_384]);
    let parsed = parselib::WavData::parse(&wav).unwrap();
    assert_eq!(parsed.channel_count, 1);
    assert_eq!(parsed.sample_rate, 48_000);
    assert_eq!(parsed.frames.len(), 3);

    let buffer = iolib::SampleBuffer::from_wav_bytes(&wav).unwrap();
    let mut source = iolib::OneShotSampleSource::new(buffer, 0.0);
    source.trigger();
    let mut mixed = vec![0.0; 6];
    source.mix_into(&mut mixed, 2, 3);
    assert!(mixed.iter().any(|sample| *sample != 0.0));
}

#[test]
fn remaining_samples_map_to_rust_state_machines() {
    let device = audio_device::AudioDevice::new(7, "speaker", true, false);
    assert_eq!(
        audio_device::select_first_output(std::slice::from_ref(&device)),
        Some(device)
    );

    let mut drums = drumthumper::DrumThumper::new(2);
    drums.load_pad(0, vec![1.0, 0.5], 1, -1.0).unwrap();
    drums.trigger(0).unwrap();
    assert_eq!(drums.render(2), vec![1.0, 0.0, 0.5, 0.0]);

    let mut player = powerplay::PowerPlayPlayer::new(2);
    player.load_track(0, vec![0.25, 0.5], 1).unwrap();
    player
        .start_playing(0, powerplay::PerformanceMode::LowLatency)
        .unwrap();
    assert_eq!(player.currently_playing_index(), Some(0));
    assert_eq!(player.render(2), vec![0.125, 0.125, 0.25, 0.25]);

    let mut game = rhythm_game::RhythmGame::new(vec![100, 200], 35);
    assert_eq!(game.tap(108), rhythm_game::TapResult::Good);
    assert_eq!(game.tap(150), rhythm_game::TapResult::Early);
}
