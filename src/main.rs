const GAMES: &str = include_str!("../answers.txt");

fn main() {
    let w = rogerthat::Wordle::new();
    for answer in GAMES.split_whitespace() {
        let guesser = rogerthat::algorithms::Naive::new();
        if let Some(score) = w.play(answer, guesser) {
            println!("{}", score);
        } else {
            eprintln!("failed to guess");
        }
    }

    println!("Hello, world!");
}
