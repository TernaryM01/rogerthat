use std::{borrow::Cow, collections::HashSet};

pub mod algorithms;

pub type Word = [u8; 5];

const DICTIONARY: &str = include_str!("../dictionary.txt");

pub struct Wordle {
    dictionary: HashSet<&'static Word>,
}

impl Wordle {
    pub fn new() -> Self {
        Self {
            dictionary: HashSet::from_iter(DICTIONARY.lines().map(|line| {
                line.split_once(' ')
                    .expect("Every line must be of the format: word + space + frequency")
                    .0
                    .as_bytes()
                    .try_into()
                    .expect("Every word should consist of 5 characters")
            })),
        }
    }

    pub fn play<G: Guesser>(&self, answer: &Word, mut guesser: G) -> Option<usize> {
        let mut history = Vec::new();
        for i in 1..=100 {
            let guess = guesser.guess(&history);
            // println!("Guessing {}", guess);
            if guess == *answer {
                println!(
                    "Guessed {}, which is the answer.",
                    std::str::from_utf8(&guess).unwrap()
                );
                return Some(i);
            }

            assert!(self.dictionary.contains(&guess));
            let correctness = Correctness::compute(answer, &guess);
            // println!("{}", Correctness::to_string(correctness));
            println!(
                "Guessed {:?}, received pattern {}.",
                std::str::from_utf8(&guess).unwrap(),
                Correctness::to_string(correctness)
            );
            history.push(Guess {
                word: Cow::Owned(guess),
                mask: correctness,
            });
        }
        None
    }
}

impl Correctness {
    fn is_misplaced(&letter: &u8, &answer: &Word, used: &mut [bool; 5]) -> bool {
        answer.iter().zip(used.iter_mut()).any(|(a, u)| {
            if *a == letter && !*u {
                *u = true;
                return true;
            }
            false
        })
    }

    fn compute(answer: &Word, guess: &Word) -> [Self; 5] {
        assert_eq!(answer.len(), 5);
        assert_eq!(guess.len(), 5);

        // Initialize as all gray
        let mut mask = [Correctness::Wrong; 5];

        // Mark things green
        let mut used = [false; 5];
        for (i, (a, g)) in answer.iter().zip(guess.iter()).enumerate() {
            if a == g {
                mask[i] = Correctness::Correct;
                used[i] = true;
            }
        }

        // Mark things yellow
        for (i, g) in guess.iter().enumerate() {
            if mask[i] == Correctness::Correct {
                // Already marked as green
                continue;
            }
            if Self::is_misplaced(&g, &answer, &mut used) {
                mask[i] = Correctness::Misplaced;
            }
        }

        mask
    }

    fn to_string(pattern: [Self; 5]) -> String {
        let mut res = String::with_capacity(5);
        for m in pattern {
            match m {
                Correctness::Correct => res.push('C'),
                Correctness::Misplaced => res.push('M'),
                Correctness::Wrong => res.push('W'),
            };
        }
        res
    }

    // generate all correctness patterns
    pub fn all_patterns() -> impl Iterator<Item = [Self; 5]> {
        itertools::iproduct!(
            [Self::Correct, Self::Misplaced, Self::Wrong],
            [Self::Correct, Self::Misplaced, Self::Wrong],
            [Self::Correct, Self::Misplaced, Self::Wrong],
            [Self::Correct, Self::Misplaced, Self::Wrong],
            [Self::Correct, Self::Misplaced, Self::Wrong]
        )
        .map(|(a, b, c, d, e)| [a, b, c, d, e])
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

pub struct Guess<'a> {
    pub word: Cow<'a, Word>,
    pub mask: [Correctness; 5],
}

impl Guess<'_> {
    pub fn matches(&self, other_word: &Word) -> bool {
        // // This one also works, but slower because it lacks short-circuiting:
        // return Correctness::compute(other_word, &self.word) == self.mask;

        assert_eq!(self.word.len(), 5);
        assert_eq!(other_word.len(), 5);

        // Check green marks
        let mut used = [false; 5];
        for (i, (g, o)) in self.word.iter().zip(other_word.iter()).enumerate() {
            if g == o {
                if self.mask[i] != Correctness::Correct {
                    return false;
                }
                used[i] = true;
            } else if self.mask[i] == Correctness::Correct {
                return false;
            }
        }

        // Check yellow marks
        for (g, m) in self.word.iter().zip(self.mask.iter()) {
            if *m == Correctness::Correct {
                // Already checked for green mark
                continue;
            }
            if Correctness::is_misplaced(&g, &other_word, &mut used)
                != (*m == Correctness::Misplaced)
            {
                return false;
            }
        }

        // The rest are all correctly marked gray
        true
    }
}

pub trait Guesser {
    fn guess(&mut self, history: &[Guess]) -> Word;
}

impl Guesser for fn(history: &[Guess]) -> Word {
    fn guess(&mut self, history: &[Guess]) -> Word {
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
            fn guess(&mut self, $history: &[Guess]) -> $crate::Word {
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
            let guesser = guesser!(|_history| { *b"right" });
            assert_eq!(w.play(b"right", guesser), Some(1));
        }
        #[test]
        fn magnificent() {
            let w = Wordle::new();
            let guesser = guesser!(|history| {
                if history.len() == 1 {
                    return *b"right";
                }
                *b"wrong"
            });
            assert_eq!(w.play(b"right", guesser), Some(2));
        }
        #[test]
        fn impressive() {
            let w = Wordle::new();
            let guesser = guesser!(|history| {
                if history.len() == 2 {
                    return *b"right";
                }
                *b"wrong"
            });
            assert_eq!(w.play(b"right", guesser), Some(3));
        }
        #[test]
        fn splendid() {
            let w = Wordle::new();
            let guesser = guesser!(|history| {
                if history.len() == 3 {
                    return *b"right";
                }
                *b"wrong"
            });
            assert_eq!(w.play(b"right", guesser), Some(4));
        }
        #[test]
        fn great() {
            let w = Wordle::new();
            let guesser = guesser!(|history| {
                if history.len() == 4 {
                    return *b"right";
                }
                *b"wrong"
            });
            assert_eq!(w.play(b"right", guesser), Some(5));
        }
        #[test]
        fn phew() {
            let w = Wordle::new();
            let guesser = guesser!(|history| {
                if history.len() == 5 {
                    return *b"right";
                }
                *b"wrong"
            });
            assert_eq!(w.play(b"right", guesser), Some(6));
        }
        #[test]
        fn oops() {
            let w = Wordle::new();
            let guesser = guesser!(|_history| { *b"wrong" });
            assert_eq!(w.play(b"right", guesser), None);
        }
    }

    mod compute {
        use crate::Correctness;

        #[test]
        fn all_green() {
            assert_eq!(Correctness::compute(b"abcde", b"abcde"), mask!(C C C C C));
        }
        #[test]
        fn all_gray() {
            assert_eq!(Correctness::compute(b"abcde", b"fghij"), mask!(W W W W W));
        }
        #[test]
        fn all_yellow() {
            assert_eq!(Correctness::compute(b"abcde", b"eabcd"), mask!(M M M M M));
        }
        #[test]
        fn repeat_green() {
            assert_eq!(Correctness::compute(b"aabbb", b"aaccc"), mask!(C C W W W));
        }
        #[test]
        fn repeat_yellow() {
            assert_eq!(Correctness::compute(b"aabbb", b"ccaac"), mask!(W W M M W));
        }
        #[test]
        fn repeat_some_green() {
            assert_eq!(Correctness::compute(b"aabbb", b"caacc"), mask!(W C M W W));
        }
        #[test]
        fn decent_chat() {
            assert_eq!(Correctness::compute(b"azzaz", b"aaabb"), mask!(C M W W W));
        }
        #[test]
        fn dumb_chat() {
            assert_eq!(Correctness::compute(b"abcde", b"aacde"), mask!(C W C C C));
        }
        #[test]
        fn ambiguous() {
            assert_eq!(Correctness::compute(b"ccaca", b"aabba"), mask!(M W W W C));
            // The following mask should really match as well.
            assert_ne!(Correctness::compute(b"ccaca", b"aabba"), mask!(W M W W C));
        }
    }
    mod guess_matcher {
        use crate::Guess;
        use std::borrow::Cow;

        macro_rules! check {
            ($prev:literal + [$($mask:tt)+] allows $next:literal) => {
                assert!(Guess {
                    word: Cow::Borrowed($prev),
                    mask: mask![$($mask )+]
                }
                .matches($next));
                assert_eq!($crate::Correctness::compute($next, $prev), mask![$($mask )+]);
            };
            ($prev:literal + [$($mask:tt)+] disallows $next:literal) => {
                assert!(!Guess {
                    word: Cow::Borrowed($prev),
                    mask: mask![$($mask )+]
                }
                .matches($next));
                assert_ne!($crate::Correctness::compute($next, $prev), mask![$($mask )+]);
            }
        }

        #[test]
        fn from_jon() {
            check!(b"abcde" + [C C C C C] allows b"abcde");
            check!(b"abcdf" + [C C C C C] disallows b"abcde");
            check!(b"abcde" + [W W W W W] allows b"fghij");
            check!(b"abcde" + [M M M M M] allows b"eabcd");
            check!(b"baaaa" + [W C M W W] allows b"aaccc");
            check!(b"baaaa" + [W C M W W] disallows b"caacc");
        }

        #[test]
        fn from_crash() {
            check!(b"tares" + [W M M W W] disallows b"brink");
        }

        #[test]
        fn from_yukosgiti() {
            check!(b"aaaab" + [C C C W M] allows b"aaabc");
            check!(b"aaabc" + [C C C M W] allows b"aaaab");
        }

        #[test]
        fn from_chat() {
            // flocular
            check!(b"aaabb" + [C M W W W] disallows b"accaa");
            // ritoban
            check!(b"abcde" + [W W W W W] disallows b"bcdea");
        }

        #[test]
        fn ambiguous() {
            check!(b"aabba" + [M W W W C] allows b"ccaca");
            // The following should really allow.
            check!(b"aabba" + [W M W W C] disallows b"ccaca");
        }
    }
}
