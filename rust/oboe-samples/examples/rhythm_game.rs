use oboe_samples::rhythm_game::RhythmGame;

fn main() {
    let mut game = RhythmGame::new(vec![100, 200], 35);
    println!("{:?}", game.tap(108));
    println!("{:?}", game.tap(150));
}
