use chrono::{self, Datelike, TimeZone};
use rand::prelude::*;
use reqwest::blocking::get;
use serde::{Deserialize, Serialize};
use serde_json;
use words::WORDLE_WORDS;
mod words;
use clap::Parser;
use std::{
    collections::HashMap,
    io::{self, Write},
};

const OLDEST: i32 = 1668726000;
const ENCOURAGE: &[&str] = &[
    "Great job!",
    "Nice!",
    "Hell yeah!",
    "Solid,",
    "Damn right!",
    "Clean,",
];

#[derive(Parser, Debug)]
#[command(version, about)]
/// Infinite wordle, written in Rust
struct Args {
    #[arg(short, long, default_value_t = 6)]
    /// Maximum attempts before failing
    guesses: i32,

    #[arg(short, long)]
    /// Add debug output. CHEAT
    debug: bool,
}

#[derive(Serialize, Deserialize, Debug)]
struct WordleResponse {
    id: u32,
    solution: String,
    print_date: String,
    days_since_launch: u32,
    editor: String,
}

fn call_wordle(timestr: String) -> Result<WordleResponse, String> {
    let res = get(format!(
        "https://www.nytimes.com/svc/wordle/v2/{}.json",
        timestr
    ))
    .expect("Failed to send request.");
    if res.status().is_success() {
        let body = res.text().expect("Failed to read response");
        let json: WordleResponse = serde_json::from_str(&body).expect("Failed to parse JSON");
        Ok(json)
    } else {
        Err("Something went wrong...".to_string())
    }
}

fn input(prompt: &str) -> String {
    let mut input = String::new();
    print!("{}", prompt);
    let _ = io::stdout().flush();
    io::stdin()
        .read_line(&mut input)
        .expect("Unable to read user input");
    input.trim().to_string()
}

#[derive(Debug, PartialEq)]
enum KeyState {
    Unknown,
    Incorrect,
    Misplaced,
    Correct,
}

fn main() {
    let args = Args::parse();

    loop {
        let mut letters: HashMap<char, KeyState> = HashMap::new();

        for letter in 'a'..='z' {
            letters.insert(letter, KeyState::Unknown);
        }

        // loop here
        let mut rng = rand::rng();
        let randtime = chrono::Utc
            .timestamp_opt(
                rng.random_range(OLDEST..chrono::Utc::now().timestamp() as i32)
                    .into(),
                0,
            )
            .unwrap();
        let timestr: String = format!(
            "{}-{}-{}",
            randtime.year(),
            format!("{:0>2}", randtime.month()),
            format!("{:0>2}", randtime.day()),
        );
        let wordle: WordleResponse =
            call_wordle(timestr.clone()).expect("Error getting today's wordle");

        let encouragement = ENCOURAGE[rng.random_range(0..ENCOURAGE.len())];

        println!("Wordle #{} by {}", wordle.id, wordle.editor);
        if args.debug {
            dbg!(&wordle);
        }
        let mut attempt = 1;
        loop {
            if attempt > args.guesses {
                println!("Solution was {}", wordle.solution);
                break;
            }
            let mut correctlets: Vec<String> = letters
                .iter()
                .filter_map(|(key, value)| {
                    if value == &KeyState::Correct {
                        Some(key.to_string())
                    } else {
                        None
                    }
                })
                .collect();
            let mut misplacedlets: Vec<String> = letters
                .iter()
                .filter_map(|(key, value)| {
                    if value == &KeyState::Misplaced {
                        Some(key.to_string())
                    } else {
                        None
                    }
                })
                .collect();
            let mut unknownlets: Vec<String> = letters
                .iter()
                .filter_map(|(key, value)| {
                    if value == &KeyState::Unknown {
                        Some(key.to_string())
                    } else {
                        None
                    }
                })
                .collect();

            correctlets.sort();
            misplacedlets.sort();
            unknownlets.sort();

            print!(
                "\x1b[2K\x1b[11G\x1b[32m{}\x1b[33m{}\x1b[0m{}",
                correctlets.join(""),
                misplacedlets.join(""),
                unknownlets.join("")
            );
            let inp = input(format!("\x1b[0G{} > ", attempt).as_str());
            if inp.len() < 5 {
                print!("\x1b[2KToo short!\x1b[1F");
                continue;
            }
            if inp.len() > 5 {
                print!("\x1b[2KToo long!\x1b[1F");
                continue;
            }
            if !WORDLE_WORDS.contains(&inp.as_str()) {
                print!("\x1b[2KNot a word!\x1b[1F");
                continue;
            }
            attempt += 1;
            print!("\x1b[2K");
            if inp.eq_ignore_ascii_case(&wordle.solution.as_str()) {
                println!(
                    "\x1b[1F\x1b[2K{} > \x1b[32m{}\x1b[0m\n{} took {} tr{}!",
                    attempt - 1,
                    inp,
                    encouragement,
                    attempt - 1,
                    if attempt - 1 == 1 { "y" } else { "ies" }
                );
                break;
            }

            print!("\x1b[1F\x1b[2K{} > ", attempt - 1);

            for ind in 0..5 {
                let curchar = inp.chars().nth(ind);
                if curchar == wordle.solution.chars().nth(ind) {
                    print!("\x1b[32m{}", curchar.unwrap());
                    if let Some(state) = letters.get_mut(&curchar.unwrap()) {
                        *state = KeyState::Correct;
                    }
                } else if wordle.solution.contains(curchar.unwrap()) {
                    print!("\x1b[33m{}", curchar.unwrap());
                    if let Some(state) = letters.get_mut(&curchar.unwrap()) {
                        if !(*state == KeyState::Correct) {
                            *state = KeyState::Misplaced;
                        }
                    }
                } else {
                    print!("\x1b[31m{}", curchar.unwrap());
                    if let Some(state) = letters.get_mut(&curchar.unwrap()) {
                        *state = KeyState::Incorrect;
                    }
                }
            }
            println!("\x1b[0m");
        }
    }
}
