use crate::{nice_print, to_word, Correctness, Guess, Guesser, Word, DICTIONARY};
use ascii::ToAsciiChar;
use std::collections::HashMap;

use ndarray::Array5;

pub struct Interactive {
    initial: HashMap<Word, usize>,
    remaining: HashMap<Word, usize>,
    hard: bool,
    use_memo: bool,
}

impl Interactive {
    pub fn new() -> Self {
        let initial = HashMap::from_iter(DICTIONARY.lines().map(|line| {
            let (word, count) = line
                .split_once(' ')
                .expect("Every line must be of the format: word + space + frequency");
            let word: [char; 5] = word.chars().collect::<Vec<char>>().try_into().unwrap();
            let count: usize = count.parse().expect("Every count should be a number");
            (word.map(|c| c.to_ascii_char().unwrap()), count)
        }));
        let remaining = initial.clone();
        Self {
            initial,
            remaining,
            hard: false,
            use_memo: true,
        }
    }

    pub fn remove(&mut self, word: &Word) {
        self.initial.remove(word);
        self.remaining.remove(word);
        self.use_memo = false;
        println!(
            "Adjusted to the fact that {} is not allowed.",
            nice_print(*word)
        );
    }

    pub fn eliminate(&mut self, word: &Word) {
        self.remaining.remove(word);
        self.use_memo = false;
        println!(
            "Adjusted to the assumption that {} is not the answer.",
            nice_print(*word)
        );
    }

    pub fn add(&mut self, word: &Word) {
        if !self.initial.contains_key(word) {
            self.initial.insert(*word, 1);
            self.use_memo = false;
        }
        println!(
            "Adjusted to the fact that {} is allowed.",
            nice_print(*word)
        );
    }

    pub fn consider(&mut self, word: &Word) {
        if let Some(goodness) = self.initial.get(word) {
            if !self.remaining.contains_key(word) {
                self.remaining.insert(*word, *goodness);
                self.use_memo = false;
            }
        } else {
            self.initial.insert(*word, 1);
            self.remaining.insert(*word, 1);
            self.use_memo = false;
        }
        println!(
            "Adjusted to the assumption that {} might be the answer.",
            nice_print(*word)
        );
    }

    pub fn remaining(&self) {
        for (&word, _) in &self.remaining {
            print!("{} ", nice_print(word));
        }
    }

    pub fn hard(&mut self) {
        self.hard = true;
    }
}

#[derive(Debug, Clone, Copy)]
struct Candidate {
    word: Word,
    goodness: f64,
}

impl Guesser for Interactive {
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
            if self.use_memo {
                return to_word("tares");
            }
        }

        if self.hard {
            self.initial = self.remaining.clone();
        }

        let remaining_count: usize = self.remaining.iter().map(|(_, &c)| c).sum();

        let mut best: Option<Candidate> = None;
        for (&word, _) in &self.initial {
            // measure goodness, which is the expected value of the information
            // - SUM_i p_i * log_2(p_i)

            let mut mask_buckets = Array5::<usize>::zeros((3, 3, 3, 3, 3));
            for (candidate, count) in &self.remaining {
                let mask = Correctness::compute(candidate, &word);
                mask_buckets[[
                    mask[0] as usize,
                    mask[1] as usize,
                    mask[2] as usize,
                    mask[3] as usize,
                    mask[4] as usize,
                ]] += count;
            }

            let mut goodness = 0.0;
            for mask in Correctness::all_patterns() {
                let in_pattern_total = mask_buckets[[
                    mask[0] as usize,
                    mask[1] as usize,
                    mask[2] as usize,
                    mask[3] as usize,
                    mask[4] as usize,
                ]];
                if in_pattern_total == 0 {
                    // avoid indeterminate arithmetic (NaN) which should evaluate to 0
                    continue;
                }
                let prob_of_pattern = (in_pattern_total as f64) / (remaining_count as f64);
                goodness -= prob_of_pattern * prob_of_pattern.log2();
            }

            if let Some(c) = best {
                use crate::EPSILON;

                // Is this one better?
                if goodness > c.goodness + EPSILON {
                    best = Some(Candidate { word, goodness });

                // Tie is pretty common when there are few words left.
                // Make sure to handle this situation well.
                } else if !(c.goodness > goodness + EPSILON) {
                    // disfavor a word that has been ruled out
                    // if neither word has been ruled out, favor the more common one
                    if !self.remaining.contains_key(&c.word)
                        || ((self.remaining.contains_key(&word))
                            && (self.initial.get(&word) > self.initial.get(&c.word)))
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
