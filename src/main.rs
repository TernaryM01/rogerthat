use clap::{Parser, ValueEnum};
use rogerthat::Guesser;

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
}

// impl std::str::FromStr for Implementation {
//     type Err = String;
//     fn from_str(arg: &str) -> Result<Self, Self::Err> {
//         match arg {
//             "naive" => Ok(Self::Naive),
//             _ => Err(format!("unknown implementation '{}'", arg)),
//         }
//     }
// }

const GAMES: &str = include_str!("../answers.txt");

fn main() {
    let cli = Cli::parse();

    match cli.implementation {
        Implementation::Naive => play(
            || rogerthat::algorithms::Naive::new(),
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
    let w = rogerthat::Wordle::new();
    for answer in GAMES
        .split_whitespace()
        .skip(skipped_rounds.unwrap_or(0))
        .take(num_rounds.unwrap_or(usize::MAX))
    {
        let guesser = (mk)();
        let answer_b: rogerthat::Word = answer.as_bytes().try_into().expect("");
        if let Some(score) = w.play(answer_b, guesser) {
            println!("The answer is {}, took {} tries.", answer, score);
        } else {
            eprintln!("failed to guess");
        }
    }
}
