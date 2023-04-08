use crate::{nice_print, Correctness, Guess, Guesser, Word};
use ascii::{AsciiChar, ToAsciiChar};
use std::io::stdin;

pub fn error_unrecognized() {
    println!("Error: Command not recognized.");
}

pub enum CmdToken {
    Unrecognized,
    Remove,
    Eliminate,
    Remaining,
    Hard,
    Word(Word),
    Mask([Correctness; 5]),
}

pub fn parse_cmd(cmd: &str) -> CmdToken {
    if cmd == "REMOVE" {
        CmdToken::Remove
    } else if cmd == "ELIMINATE" {
        CmdToken::Eliminate
    } else if cmd == "REMAINING" {
        CmdToken::Remaining
    } else if cmd == "HARD" {
        CmdToken::Hard
    } else if cmd.len() == 5 {
        let identifier = cmd.chars().next().unwrap();
        if identifier == '-' || identifier == '#' || identifier == '+' {
            // cmd is a mask pattern.
            let mut mask = Vec::<Correctness>::with_capacity(5);
            for char in cmd.chars() {
                match char {
                    '-' => {
                        mask.push(Correctness::Wrong);
                    }
                    '#' => {
                        mask.push(Correctness::Correct);
                    }
                    '+' => {
                        mask.push(Correctness::Misplaced);
                    }
                    _ => {
                        return CmdToken::Unrecognized;
                    }
                };
            }
            let mask = mask.try_into().unwrap();
            CmdToken::Mask(mask)
        } else {
            // cmd is a word.
            let mut word = Vec::<AsciiChar>::with_capacity(5);
            for char in cmd.chars() {
                if let Ok(char) = char.to_ascii_char() {
                    word.push(char);
                } else {
                    return CmdToken::Unrecognized;
                }
            }
            let word = word.try_into().unwrap();
            CmdToken::Word(word)
        }
    } else {
        CmdToken::Unrecognized
    }
}

pub fn interactive() {
    println!("Type history. Each line is: word + space + pattern.");
    println!("'-' for Wrong/Gray, '#' for Correct/Green, '+' for Misplaced/Yellow.");
    println!("If the suggestion is not allowed, type 'REMOVE'.");
    println!("You can also manually remove any word by typing 'REMOVE' + space + word.");
    println!("If a word is allowed but you don't think it's the answer, use 'ELIMINATE' instead of 'REMOVE'.");
    println!("To allow a word without considering it as possibly the answer (the opposite of and undoes 'REMOVE'), use 'ALLOW'.");
    println!("To allow a word and consider it as possibly the answer (the opposite of and undoes 'ELIMINATE'), use 'CONSIDER'.");
    println!("If you follow the suggestion, you can just type the pattern, omitting the word (and the space).");
    println!("To list all remaining possible words, type 'REMAINING'.");
    println!("To enter hard mode, type 'HARD'.");

    let mut guesser = crate::algorithms::Interactive::new();
    let mut history = Vec::<Guess>::new();

    loop {
        let guess = guesser.guess(&history);
        println!("Suggested guess is: {}", nice_print(guess));

        let mut user_input = String::new();
        let buzz = stdin();
        buzz.read_line(&mut user_input).unwrap();
        let mut user_slice_iter = user_input.trim().split_whitespace();

        let arg1 = user_slice_iter.next();
        let arg2 = user_slice_iter.next();

        if let Some(arg1) = arg1 {
            let arg1 = parse_cmd(arg1);
            match arg1 {
                CmdToken::Remove => {
                    if let Some(arg2) = arg2 {
                        if let CmdToken::Word(word) = parse_cmd(arg2) {
                            guesser.remove(&word);
                            continue;
                        }
                    } else {
                        guesser.remove(&guess);
                        continue;
                    }
                }
                CmdToken::Eliminate => {
                    if let Some(arg2) = arg2 {
                        if let CmdToken::Word(word) = parse_cmd(arg2) {
                            guesser.eliminate(&word);
                            continue;
                        }
                    } else {
                        guesser.eliminate(&guess);
                        continue;
                    }
                }
                CmdToken::Remaining => {
                    guesser.remaining();
                    continue;
                }
                CmdToken::Hard => {
                    guesser.hard();
                    println!("Hard mode activated.");
                    continue;
                }
                CmdToken::Mask(mask) => {
                    history.push(Guess { word: guess, mask });
                    continue;
                }
                CmdToken::Word(word) => {
                    if let Some(arg2) = arg2 {
                        if let CmdToken::Mask(mask) = parse_cmd(arg2) {
                            history.push(Guess { word, mask });
                            continue;
                        }
                    }
                }
                _ => {}
            }
        }
        error_unrecognized();
    }
}
