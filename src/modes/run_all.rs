use crate::{to_word, Guesser, Word, Wordle, GAMES};
use colored::Colorize;

pub fn run_all<G: Guesser>(
    mut mk: impl FnMut() -> G,
    num_rounds: Option<usize>,
    skipped_rounds: Option<usize>,
) {
    let w = Wordle::new();
    let mut guesser = (mk)();
    for answer in GAMES
        .split_whitespace()
        .skip(skipped_rounds.unwrap_or(0))
        .take(num_rounds.unwrap_or(10))
    {
        println!("{}", "New game".blue());

        let answer_b: Word = to_word(answer);
        if let Some(score) = w.play(&answer_b, &mut guesser) {
            println!(
                "The answer is '{}', took {} tries.",
                answer.to_uppercase().blue(),
                score.to_string().blue().bold()
            );
        } else {
            eprintln!("failed to guess");
        }
    }
}
