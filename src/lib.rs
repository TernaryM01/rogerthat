use std::collections::HashSet;

pub mod algorithms;

const DICTIONARY: &str = include_str!("../dictionary.txt");

pub struct Wordle {
    dictionary: HashSet<&'static str>,
}

impl Wordle {
    pub fn new() -> Self {
        Self {
            dictionary: HashSet::from_iter(DICTIONARY.lines().map(|line| {
                line.split_once(' ')
                    .expect("Every line must be of the format: word + space + frequency")
                    .0
            })),
        }
    }

    pub fn play<G: Guesser>(&self, answer: &'static str, mut guesser: G) -> Option<usize> {
        let mut history = Vec::new();
        for i in 1..=100 {
            let guess = guesser.guess(&history[..]);
            if guess == answer {
                return Some(i);
            }

            assert!(self.dictionary.contains(&*guess));
            let correctness = Correctness::compute(answer, &guess);
            history.push(Guess {
                word: guess,
                mask: correctness,
            });
        }
        None
    }
}

impl Correctness {
    fn is_misplaced(letter: char, answer: &str, used: &mut [bool; 5]) -> bool {
        answer.chars().enumerate().any(|(i, a)| {
            if a == letter && !used[i] {
                used[i] = true;
                return true;
            }
            false
        })
    }

    fn compute(answer: &str, guess: &str) -> [Self; 5] {
        assert_eq!(answer.len(), 5);
        assert_eq!(guess.len(), 5);

        // Initialize as all gray
        let mut c = [Correctness::Wrong; 5];

        //Mark things green
        let mut used = [false; 5];
        for (i, (a, g)) in answer.chars().zip(guess.chars()).enumerate() {
            if a == g {
                c[i] = Correctness::Correct;
                used[i] = true;
            }
        }

        //Mark things yellow
        for (i, g) in guess.chars().enumerate() {
            if c[i] == Correctness::Correct {
                // Already marked as green
                continue;
            }
            if Self::is_misplaced(g, answer, &mut used) {
                c[i] = Correctness::Misplaced;
            }
        }

        c
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Correctness {
    // Green
    Correct,
    // Yellow
    Misplaced,
    // Gray
    Wrong,
}

pub struct Guess {
    pub word: String,
    pub mask: [Correctness; 5],
}

impl Guess {
    pub fn matches(&self, other_word: &str) -> bool {
        assert_eq!(self.word.len(), 5);
        assert_eq!(other_word.len(), 5);

        let mut used = [false; 5];
        for (i, ((g, &mask), o)) in self
            .word
            .chars()
            .zip(&self.mask)
            .zip(other_word.chars())
            .enumerate()
        {
            match mask {
                Correctness::Correct => {
                    if g != o {
                        return false;
                    }
                    used[i] = true;
                }
                Correctness::Misplaced => {
                    if g == o {
                        return false;
                    }
                }
                Correctness::Wrong => {
                    if g == o {
                        return false;
                    }
                }
            }
        }
        todo!()
    }
}

pub trait Guesser {
    fn guess(&mut self, history: &[Guess]) -> String;
}

impl Guesser for fn(history: &[Guess]) -> String {
    fn guess(&mut self, history: &[Guess]) -> String {
        (*self)(history)
    }
}

#[cfg(test)]
macro_rules! mask {
    (C) => {$crate::Correctness::Correct};
    (M) => {$crate::Correctness::Misplaced};
    (W) => {$crate::Correctness::Wrong};
    ($($c:tt)+) => {[
        $(mask!($c)),+
    ]}
}

#[cfg(test)]
macro_rules! guesser {
    (|$history:ident| $impl:block) => {{
        struct G;
        impl $crate::Guesser for G {
            fn guess(&mut self, $history: &[Guess]) -> String {
                $impl
            }
        }
        G
    }};
}

#[cfg(test)]
mod tests {
    mod game {
        use crate::{Guess, Wordle};

        #[test]
        fn genius() {
            let w = Wordle::new();
            let guesser = guesser!(|_history| { "right".to_string() });
            assert_eq!(w.play("right", guesser), Some(1));
        }
        #[test]
        fn magnificent() {
            let w = Wordle::new();
            let guesser = guesser!(|history| {
                if history.len() == 1 {
                    return "right".to_string();
                }
                "wrong".to_string()
            });
            assert_eq!(w.play("right", guesser), Some(2));
        }
        #[test]
        fn impressive() {
            let w = Wordle::new();
            let guesser = guesser!(|history| {
                if history.len() == 2 {
                    return "right".to_string();
                }
                "wrong".to_string()
            });
            assert_eq!(w.play("right", guesser), Some(3));
        }
        #[test]
        fn splendid() {
            let w = Wordle::new();
            let guesser = guesser!(|history| {
                if history.len() == 3 {
                    return "right".to_string();
                }
                "wrong".to_string()
            });
            assert_eq!(w.play("right", guesser), Some(4));
        }
        #[test]
        fn great() {
            let w = Wordle::new();
            let guesser = guesser!(|history| {
                if history.len() == 4 {
                    return "right".to_string();
                }
                "wrong".to_string()
            });
            assert_eq!(w.play("right", guesser), Some(5));
        }
        #[test]
        fn phew() {
            let w = Wordle::new();
            let guesser = guesser!(|history| {
                if history.len() == 5 {
                    return "right".to_string();
                }
                "wrong".to_string()
            });
            assert_eq!(w.play("right", guesser), Some(6));
        }
        #[test]
        fn oops() {
            let w = Wordle::new();
            let guesser = guesser!(|_history| { "wrong".to_string() });
            assert_eq!(w.play("right", guesser), None);
        }
    }

    mod compute {
        use crate::Correctness;

        #[test]
        fn all_green() {
            assert_eq!(Correctness::compute("abcde", "abcde"), mask!(C C C C C));
        }
        #[test]
        fn all_gray() {
            assert_eq!(Correctness::compute("abcde", "fghij"), mask!(W W W W W));
        }
        #[test]
        fn all_yellow() {
            assert_eq!(Correctness::compute("abcde", "eabcd"), mask!(M M M M M));
        }
        #[test]
        fn repeat_green() {
            assert_eq!(Correctness::compute("aabbb", "aaccc"), mask!(C C W W W));
        }
        #[test]
        fn repeat_yellow() {
            assert_eq!(Correctness::compute("aabbb", "ccaac"), mask!(W W M M W));
        }
        #[test]
        fn repeat_some_green() {
            assert_eq!(Correctness::compute("aabbb", "caacc"), mask!(W C M W W));
        }
        #[test]
        fn decent_chat() {
            assert_eq!(Correctness::compute("azzaz", "aaabb"), mask!(C M W W W));
        }
        #[test]
        fn dumb_chat() {
            assert_eq!(Correctness::compute("abcde", "aacde"), mask!(C W C C C));
        }
        #[test]
        fn smart_ass() {
            assert_eq!(Correctness::compute("ccaca", "aabba"), mask!(M W W W C));
        }
    }
}
