use oboe_samples::powerplay::{PerformanceMode, PowerPlayPlayer};

fn main() {
    let mut player = PowerPlayPlayer::new(2);
    player.load_track(0, vec![0.25, 0.5], 1).unwrap();
    player
        .start_playing(0, PerformanceMode::LowLatency)
        .unwrap();
    println!("{:?}", player.render(2));
}
