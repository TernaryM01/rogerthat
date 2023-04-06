use crate::{to_word, Correctness, Guess, Guesser, Word, DICTIONARY};
use ascii::ToAsciiChar;
use std::collections::HashMap;

pub struct Naive {
    initial: HashMap<Word, usize>,
    remaining: HashMap<Word, usize>,
}

impl Naive {
    pub fn new() -> Self {
        let mut res = Self {
            initial: HashMap::from_iter(DICTIONARY.lines().map(|line| {
                let (word, count) = line
                    .split_once(' ')
                    .expect("Every line must be of the format: word + space + frequency");
                let word: [char; 5] = word.chars().collect::<Vec<char>>().try_into().unwrap();
                let count: usize = count.parse().expect("Every count should be a number");
                (word.map(|c| c.to_ascii_char().unwrap()), count)
            })),
            remaining: HashMap::new(),
        };
        res.remaining = res.initial.clone();
        res
    }
}

#[derive(Debug, Clone, Copy)]
struct Candidate {
    word: Word,
    goodness: f64,
}

impl Guesser for Naive {
    fn guess(&mut self, history: &[Guess]) -> Word {
        if let Some(last) = history.last() {
            self.remaining.retain(|word, _| last.matches(word));

            let num_remains = self.remaining.len();
            println!("Number of remaining possibilities: {}", num_remains);
            // If only 1 possibility remains, return that as the guess.
            // This is essential, because otherwise,
            // any guess would be considered to be as good as any other.
            if num_remains == 1 {
                return *self.remaining.iter().next().unwrap().0;
            }
        } else {
            // First guess
            self.remaining = self.initial.clone();
            return to_word("tares");
        }

        let remaining_count: usize = self.remaining.iter().map(|(_, &c)| c).sum();

        let mut best: Option<Candidate> = None;
        for (&word, _) in &self.initial {
            // measure goodness, which is the expected value of the information
            // - SUM_i p_i * log_2(p_i)
            let mut goodness = 0.0;
            for pattern in Correctness::all_patterns() {
                let mut in_pattern_total = 0;
                for (candidate, count) in &self.remaining {
                    let g = Guess {
                        word,
                        mask: pattern,
                    };
                    if g.matches(candidate) {
                        in_pattern_total += count;
                    }
                }
                if in_pattern_total == 0 {
                    // avoid indeterminate arithmetic (NaN) which should evaluate to 0
                    continue;
                }
                let prob_of_pattern = in_pattern_total as f64 / remaining_count as f64;
                goodness -= prob_of_pattern * prob_of_pattern.log2();
            }

            if let Some(c) = best {
                // Is this one better?
                if goodness > c.goodness {
                    best = Some(Candidate { word, goodness })

                // Tie is pretty common when there are few words left.
                // Make sure to handle this situation well.
                } else if goodness == c.goodness {
                    // disfavor a word that has been ruled out
                    // if neither word has been ruled out, favor the more common one
                    if !self.remaining.contains_key(&c.word)
                        || (self.remaining.contains_key(&word))
                            && (self.initial.get(&word) > self.initial.get(&c.word))
                    {
                        best = Some(Candidate { word, goodness });
                    }
                }
            } else {
                best = Some(Candidate { word, goodness });
            }
        }
        best.unwrap().word
    }
}
