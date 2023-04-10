use ascii::{AsciiChar, ToAsciiChar};
use colored::{ColoredString, Colorize};
use std::collections::HashSet;

pub mod algorithms;
pub mod modes;

// TODO: Make Word a Struct instead of a type alias ======================
// Traits should then be implemented for it, such as to be used in HashMap

pub type Word = [AsciiChar; 5];

pub fn to_word(slice: &str) -> Word {
    let word: [char; 5] = slice.chars().collect::<Vec<char>>().try_into().unwrap();
    word.map(|c| c.to_ascii_char().unwrap())
}

pub fn nice_print(word: Word) -> ColoredString {
    let mut printed = String::new();
    for ch in word {
        printed.push(ch.as_char());
    }
    printed.to_uppercase().purple().bold()
}

// =======================================================================

const DICTIONARY: &str = include_str!("../dictionary.txt");
pub const GAMES: &str = include_str!("../answers.txt");
const MAX_GUESSES: usize = 100;

pub const EPSILON: f64 = 0.0000000000001;

pub struct Wordle {
    dictionary: HashSet<Word>,
}

impl Wordle {
    pub fn new() -> Self {
        Self {
            dictionary: HashSet::from_iter(DICTIONARY.lines().map(|line| {
                let word: [char; 5] = line
                    .split_once(' ')
                    .expect("Every line must be of the format: word + space + frequency")
                    .0
                    .chars()
                    .collect::<Vec<char>>()
                    .try_into()
                    .expect("Every word should consist of 5 characters");
                word.map(|c| {
                    c.to_ascii_char()
                        .expect("Only ASCII characters are allowed")
                })
            })),
        }
    }

    pub fn play<G: Guesser>(&self, answer: &Word, guesser: &mut G) -> Option<usize> {
        let mut history = Vec::new();
        for i in 1..=MAX_GUESSES {
            let guess = guesser.guess(&history);
            if guess == *answer {
                println!("Guessed '{}', which is the answer.", nice_print(guess));
                return Some(i);
            }

            assert!(self.dictionary.contains(&guess));
            let correctness = Correctness::compute(answer, &guess);
            println!(
                "Guessed '{}', received pattern: {}",
                nice_print(guess),
                Correctness::to_string(&correctness)
            );
            history.push(Guess {
                word: guess,
                mask: correctness,
            });
        }
        None
    }
}

impl Correctness {
    fn is_misplaced(letter: AsciiChar, answer: &Word, used: &mut [bool; 5]) -> bool {
        // Because all the lengths are carried by the types,
        // the compiler should be able to eliminate all redundant bounds checks!
        for i in 0..5 {
            if (answer[i] == letter) && !used[i] {
                used[i] = true;
                return true;
            }
        }
        false
    }

    fn compute(answer: &Word, guess: &Word) -> [Self; 5] {
        // Because all the lengths are carried by the types,
        // the compiler should be able to eliminate all redundant bounds checks!

        // Initialize as all gray
        let mut mask: [Correctness; 5] = [Correctness::Wrong; 5];
        let mut used: [bool; 5] = [false; 5];

        // Mark things green
        for i in 0..5 {
            if answer[i] == guess[i] {
                mask[i] = Correctness::Correct;
                used[i] = true;
            }
        }

        // Mark things yellow
        for i in 0..5 {
            if mask[i] == Correctness::Correct {
                // Already marked as green
                continue;
            }
            if Self::is_misplaced(guess[i], answer, &mut used) {
                mask[i] = Correctness::Misplaced;
            }
        }

        mask
    }

    fn to_string(pattern: &[Self; 5]) -> String {
        let mut res = String::with_capacity(5);
        for m in pattern {
            match m {
                Correctness::Correct => res.push('🟩'),
                Correctness::Misplaced => res.push('🟨'),
                Correctness::Wrong => res.push('⬛'),
            };
        }
        res
    }

    // generate all correctness patterns
    #[inline(always)]
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
    // Gray
    Wrong,
    // Yellow
    Misplaced,
    // Green
    Correct,
}

pub struct Guess {
    pub word: Word,
    pub mask: [Correctness; 5],
}

impl Guess {
    pub fn matches(&self, other_word: &Word) -> bool {
        // // This one also works, but slower because it lacks short-circuiting:
        // return Correctness::compute(other_word, self.word) == self.mask;

        // Because all the lengths are carried by the types,
        // the compiler should be able to eliminate all redundant bounds checks!

        let mut used: [bool; 5] = [false; 5];

        // Check green marks
        for i in 0..5 {
            if self.word[i] == other_word[i] {
                if self.mask[i] != Correctness::Correct {
                    return false;
                }
                used[i] = true;
            } else if self.mask[i] == Correctness::Correct {
                return false;
            }
        }

        // Check yellow marks
        for i in 0..5 {
            if self.mask[i] == Correctness::Correct {
                // Already checked for green mark
                continue;
            }
            if Correctness::is_misplaced(self.word[i], other_word, &mut used)
                != (self.mask[i] == Correctness::Misplaced)
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
        use crate::{to_word, Guess, Wordle};

        #[test]
        fn genius() {
            let w = Wordle::new();
            let mut guesser = guesser!(|_history| { to_word("right") });
            assert_eq!(w.play(&to_word("right"), &mut guesser), Some(1));
        }
        #[test]
        fn magnificent() {
            let w = Wordle::new();
            let mut guesser = guesser!(|history| {
                if history.len() == 1 {
                    return to_word("right");
                }
                to_word("wrong")
            });
            assert_eq!(w.play(&to_word("right"), &mut guesser), Some(2));
        }
        #[test]
        fn impressive() {
            let w = Wordle::new();
            let mut guesser = guesser!(|history| {
                if history.len() == 2 {
                    return to_word("right");
                }
                to_word("wrong")
            });
            assert_eq!(w.play(&to_word("right"), &mut guesser), Some(3));
        }
        #[test]
        fn splendid() {
            let w = Wordle::new();
            let mut guesser = guesser!(|history| {
                if history.len() == 3 {
                    return to_word("right");
                }
                to_word("wrong")
            });
            assert_eq!(w.play(&to_word("right"), &mut guesser), Some(4));
        }
        #[test]
        fn great() {
            let w = Wordle::new();
            let mut guesser = guesser!(|history| {
                if history.len() == 4 {
                    return to_word("right");
                }
                to_word("wrong")
            });
            assert_eq!(w.play(&to_word("right"), &mut guesser), Some(5));
        }
        #[test]
        fn phew() {
            let w = Wordle::new();
            let mut guesser = guesser!(|history| {
                if history.len() == 5 {
                    return to_word("right");
                }
                to_word("wrong")
            });
            assert_eq!(w.play(&to_word("right"), &mut guesser), Some(6));
        }
        #[test]
        fn oops() {
            let w = Wordle::new();
            let mut guesser = guesser!(|_history| { to_word("wrong") });
            assert_eq!(w.play(&to_word("right"), &mut guesser), None);
        }
    }

    mod compute {
        use crate::{to_word, Correctness};

        #[test]
        fn all_green() {
            assert_eq!(
                Correctness::compute(&to_word("abcde"), &to_word("abcde")),
                mask!(C C C C C)
            );
        }
        #[test]
        fn all_gray() {
            assert_eq!(
                Correctness::compute(&to_word("abcde"), &to_word("fghij")),
                mask!(W W W W W)
            );
        }
        #[test]
        fn all_yellow() {
            assert_eq!(
                Correctness::compute(&to_word("abcde"), &to_word("eabcd")),
                mask!(M M M M M)
            );
        }
        #[test]
        fn repeat_green() {
            assert_eq!(
                Correctness::compute(&to_word("aabbb"), &to_word("aaccc")),
                mask!(C C W W W)
            );
        }
        #[test]
        fn repeat_yellow() {
            assert_eq!(
                Correctness::compute(&to_word("aabbb"), &to_word("ccaac")),
                mask!(W W M M W)
            );
        }
        #[test]
        fn repeat_some_green() {
            assert_eq!(
                Correctness::compute(&to_word("aabbb"), &to_word("caacc")),
                mask!(W C M W W)
            );
        }
        #[test]
        fn decent_chat() {
            assert_eq!(
                Correctness::compute(&to_word("azzaz"), &to_word("aaabb")),
                mask!(C M W W W)
            );
        }
        #[test]
        fn meh_chat() {
            assert_eq!(
                Correctness::compute(&to_word("abcde"), &to_word("aacde")),
                mask!(C W C C C)
            );
        }
        #[test]
        fn ambiguous() {
            assert_eq!(
                Correctness::compute(&to_word("ccaca"), &to_word("aabba")),
                mask!(M W W W C)
            );
            // The following mask should really match as well.
            assert_ne!(
                Correctness::compute(&to_word("ccaca"), &to_word("aabba")),
                mask!(W M W W C)
            );
        }
    }
    mod guess_matcher {
        use crate::{to_word, Guess};

        macro_rules! check {
            ($prev:literal + [$($mask:tt)+] allows $next:literal) => {
                assert!(Guess {
                    word: to_word($prev),
                    mask: mask![$($mask )+]
                }
                .matches(&to_word($next)));
                assert_eq!($crate::Correctness::compute(&to_word($next), &to_word($prev)), mask![$($mask )+]);
            };
            ($prev:literal + [$($mask:tt)+] disallows $next:literal) => {
                assert!(!Guess {
                    word: to_word($prev),
                    mask: mask![$($mask )+]
                }
                .matches(&to_word($next)));
                assert_ne!($crate::Correctness::compute(&to_word($next), &to_word($prev)), mask![$($mask )+]);
            }
        }

        #[test]
        fn from_jon() {
            check!("abcde" + [C C C C C] allows "abcde");
            check!("abcdf" + [C C C C C] disallows "abcde");
            check!("abcde" + [W W W W W] allows "fghij");
            check!("abcde" + [M M M M M] allows "eabcd");
            check!("baaaa" + [W C M W W] allows "aaccc");
            check!("baaaa" + [W C M W W] disallows "caacc");
        }

        #[test]
        fn from_crash() {
            check!("tares" + [W M M W W] disallows "brink");
        }

        #[test]
        fn from_yukosgiti() {
            check!("aaaab" + [C C C W M] allows "aaabc");
            check!("aaabc" + [C C C M W] allows "aaaab");
        }

        #[test]
        fn from_chat() {
            // flocular
            check!("aaabb" + [C M W W W] disallows "accaa");
            // ritoban
            check!("abcde" + [W W W W W] disallows "bcdea");
        }

        #[test]
        fn ambiguous() {
            check!("aabba" + [M W W W C] allows "ccaca");
            // The following should really allow.
            check!("aabba" + [W M W W C] disallows "ccaca");
        }
        #[test]
        fn hard() {
            check!("zimbi" + [W C W W W] allows "hilly");
        }
    }
}
