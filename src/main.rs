use clap::{Parser, ValueEnum};
use rogerthat::{to_word, Guesser, Word, Wordle};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[arg(short, long, value_enum)]
    implementation: Implementation,

    #[clap(short, long)]
    num_rounds: Option<usize>,

    #[clap(short, long)]
    skipped_rounds: Option<usize>,
}

#[derive(Parser, Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
enum Implementation {
    Naive,
    Cached,
    MaskBuckets,
}

const GAMES: &str = include_str!("../answers.txt");

fn main() {
    let cli = Cli::parse();

    match cli.implementation {
        Implementation::Naive => play(
            || rogerthat::algorithms::Naive::new(),
            cli.num_rounds,
            cli.skipped_rounds,
        ),
        Implementation::Cached => play(
            || rogerthat::algorithms::Cached::new(),
            cli.num_rounds,
            cli.skipped_rounds,
        ),
        Implementation::MaskBuckets => play(
            || rogerthat::algorithms::MaskBuckets::new(),
            cli.num_rounds,
            cli.skipped_rounds,
        ),
    }
}

fn play<G: Guesser>(
    mut mk: impl FnMut() -> G,
    num_rounds: Option<usize>,
    skipped_rounds: Option<usize>,
) {
    let w = Wordle::new();
    for answer in GAMES
        .split_whitespace()
        .skip(skipped_rounds.unwrap_or(0))
        .take(num_rounds.unwrap_or(usize::MAX))
    {
        let mut guesser = (mk)();
        let answer_b: Word = to_word(answer);
        if let Some(score) = w.play(&answer_b, &mut guesser) {
            println!("The answer is {}, took {} tries.", answer, score);
        } else {
            eprintln!("failed to guess");
        }
    }
}
