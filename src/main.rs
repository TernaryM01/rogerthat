use clap::{Parser, ValueEnum};
use rogerthat::modes::{interactive, run_all};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[arg(short, long, value_enum)]
    mode: Option<Mode>,

    #[arg(short, long, value_enum)]
    implementation: Option<Implementation>,

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
    Memoized,
    Interactive,
}

#[derive(Parser, Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
enum Mode {
    RunAll,
    Interactive,
}

fn main() {
    let cli = Cli::parse();

    match cli.mode {
        Some(Mode::Interactive) => interactive(),
        Some(Mode::RunAll) | None => match cli.implementation {
            Some(Implementation::Naive) => run_all(
                || rogerthat::algorithms::Naive::new(),
                cli.num_rounds,
                cli.skipped_rounds,
            ),
            Some(Implementation::Cached) => run_all(
                || rogerthat::algorithms::Cached::new(),
                cli.num_rounds,
                cli.skipped_rounds,
            ),
            Some(Implementation::MaskBuckets) => run_all(
                || rogerthat::algorithms::MaskBuckets::new(),
                cli.num_rounds,
                cli.skipped_rounds,
            ),
            Some(Implementation::Memoized) | None => run_all(
                || rogerthat::algorithms::Memoized::new(),
                cli.num_rounds,
                cli.skipped_rounds,
            ),
            Some(Implementation::Interactive) => run_all(
                || rogerthat::algorithms::Interactive::new(),
                cli.num_rounds,
                cli.skipped_rounds,
            ),
        },
    }
}
