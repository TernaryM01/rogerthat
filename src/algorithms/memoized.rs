use ndarray::Array5;
use once_cell::sync::OnceCell;

use crate::{to_word, Correctness, Guess, Guesser, Word, DICTIONARY};
use ascii::ToAsciiChar;
use std::{borrow::Cow, collections::HashMap};

static INITIAL: OnceCell<HashMap<Word, usize>> = OnceCell::new();

pub struct Memoized {
    remaining: Cow<'static, HashMap<Word, usize>>,
    second_guess: [Option<Word>; 3 * 3 * 3 * 3 * 3],
}

fn get_mask_idx(mask: [Correctness; 5]) -> usize {
    (mask[0] as usize) * (3 as usize).pow(0)
        + (mask[1] as usize) * (3 as usize).pow(1)
        + (mask[2] as usize) * (3 as usize).pow(2)
        + (mask[3] as usize) * (3 as usize).pow(3)
        + (mask[4] as usize) * (3 as usize).pow(4)
}

impl Memoized {
    pub fn new() -> Self {
        Self {
            remaining: Cow::Borrowed(INITIAL.get_or_init(|| {
                HashMap::from_iter(DICTIONARY.lines().map(|line| {
                    let (word, count) = line
                        .split_once(' ')
                        .expect("Every line must be of the format: word + space + frequency");
                    let word: [char; 5] = word.chars().collect::<Vec<char>>().try_into().unwrap();
                    let count: usize = count.parse().expect("Every count should be a number");
                    (word.map(|c| c.to_ascii_char().unwrap()), count)
                }))
            })),
            second_guess: [None; 3 * 3 * 3 * 3 * 3],
        }
    }
}

#[derive(Debug, Clone, Copy)]
struct Candidate {
    word: Word,
    goodness: f64,
}

impl Guesser for Memoized {
    fn guess(&mut self, history: &[Guess]) -> Word {
        if let Some(last) = history.last() {
            self.remaining.to_mut().retain(|word, _| last.matches(word));

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
            self.remaining = Cow::Borrowed(INITIAL.get().unwrap());
            return to_word("crate");
        }

        // retrieve memoized second guess
        if history.len() == 1 {
            // println!("Second guess");
            if let Some(word) = self.second_guess[get_mask_idx(history.last().unwrap().mask)] {
                println!("I remember this!");
                return word;
            }
        }

        let remaining_count: usize = self.remaining.iter().map(|(_, &c)| c).sum();

        let mut best: Option<Candidate> = None;
        let dict = INITIAL.get().unwrap();
        for (&word, _) in dict {
            // measure goodness, which is the expected value of the information
            // - SUM_i p_i * log_2(p_i)

            let mut mask_buckets = Array5::<usize>::zeros((3, 3, 3, 3, 3));
            for (candidate, count) in &*self.remaining {
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
                // Is this one better?
                if goodness > c.goodness {
                    // println!("{} is better than {} ({} > {})", word, c.word, goodness, c.goodness);
                    best = Some(Candidate { word, goodness })

                // pretty common when there are few words left
                } else if goodness == c.goodness {
                    // disfavor a word that has been ruled out
                    // if both words haven't been ruled out, favor the more common one
                    if !self.remaining.contains_key(&c.word)
                        || (self.remaining.contains_key(&word))
                            && (dict.get(&word) > dict.get(&c.word))
                    {
                        // println!("'{}' has been ruled out", std::str::from_utf8(&c.word).unwrap());
                        best = Some(Candidate { word, goodness });
                    }
                }
            } else {
                // println!("starting with {} (goodness: {})", word, goodness);
                best = Some(Candidate { word, goodness });
            }
        }
        let guess = best.unwrap().word;

        // If this is the second guess, remember it.
        if history.len() == 1 {
            // println!("Second guess");
            self.second_guess[get_mask_idx(history.last().unwrap().mask)] = Some(guess);
        }

        guess
    }
}
