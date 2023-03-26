use crate::{Guess, Guesser, DICTIONARY};
use std::collections::HashMap;

pub struct Naive {
    remaining: HashMap<&'static str, usize>,
}

impl Naive {
    pub fn new() -> Self {
        Naive {
            remaining: HashMap::from_iter(DICTIONARY.lines().map(|line| {
                let (word, count) = line
                    .split_once(' ')
                    .expect("Every line must be of the format: word + space + frequency");
                let count: usize = count.parse().expect("Every count should be a number");
                (word, count)
            })),
        }
    }
}

#[derive(Debug, Clone, Copy)]
struct Candidate {
    word: &'static str,
    count: usize,
    goodness: f64,
}

impl Guesser for Naive {
    fn guess(&mut self, _history: &[Guess]) -> String {
        if let Some(last) = _history.last() {
            self.remaining
                .retain(|word, _| /*last.matches(word)*/ todo!());
        }

        let mut best: Option<Candidate> = None;
        for (&word, &count) in &self.remaining {
            // TODO: properly measure goodness
            let goodness = 0.0;
            if let Some(c) = best {
                // Is this one better?
                if goodness > c.goodness {
                    best = Some(Candidate {
                        word,
                        count,
                        goodness,
                    })
                }
            } else {
                best = Some(Candidate {
                    word,
                    count,
                    goodness,
                });
            }
        }
        best.unwrap().word.to_string()
    }
}
