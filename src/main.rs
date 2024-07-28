use std::process::{Command, Child};
use std::io::{self, stdin, stdout, Write, Read};
use std::env;
use crossterm::{terminal::{enable_raw_mode, disable_raw_mode}, cursor};

fn appendbuf(input: &mut String, c: char) {
    input.push(c); // Add the character to the input buffer
    print!("{}", c);
    stdout().flush().unwrap(); // Display the character as it's typed
}

fn backspace(input: &mut String) {
    if !input.is_empty() {
        input.pop(); // Remove the last character from the input buffer
        print!("{} {}", cursor::MoveLeft(1), cursor::MoveLeft(1)); // Move cursor back and clear character
        stdout().flush().unwrap();
    }
}

fn historyup(input: &mut String, path: std::path::PathBuf, history: &mut Vec<String>, historyindex: &mut usize) {
    if *historyindex != 0 {
        *historyindex -= 1;
    }
    if history.len() != 0 {
        *input = (*history[*historyindex]).to_string();
        print!("\r\x1B[K");
        print!("\r{}> ", path.display());
        print!("{input}");
        stdout().flush().unwrap();
    }
}

fn historydown(input: &mut String, path: std::path::PathBuf, history: &mut Vec<String>, historyindex: &mut usize) {
    if history.len() != 0 {
        if *historyindex != history.len() - 1 {
            *historyindex += 1;
        }
        *input = (*history[*historyindex]).to_string();
        print!("\r\x1B[K");
        print!("\r{}> ", path.display());
        print!("{input}");
        stdout().flush().unwrap();
    }
}

fn main() -> std::io::Result<()> {
    let mut history: Vec<String> = Vec::new();
    enable_raw_mode().unwrap(); // Enable raw mode for capturing input
    let mut input = String::new(); // Input buffer

    loop {
        let path: std::path::PathBuf = env::current_dir()?;
        let mut historyindex: &mut usize = &mut history.len();
        let mut escape_sequence = Vec::new();
        print!("\r{}> ", path.display());
        stdout().flush().unwrap(); // Ensure prompt is displayed

        for b in stdin().bytes() {
            let c = b.unwrap() as char;

            if !escape_sequence.is_empty() {
                escape_sequence.push(c as u8);
                if escape_sequence.len() == 3 {
                    if escape_sequence == vec![0x1B, b'[', b'A'] { // Up arrow key
                        historyup(&mut input, path.clone(), &mut history, &mut historyindex);
                    } else if escape_sequence == vec![0x1B, b'[', b'B'] { // Down arrow key
                        historydown(&mut input, path.clone(), &mut history, &mut historyindex);
                    }
                    escape_sequence.clear();
                }
            } else if c == '\x1B' { // Start of an escape sequence
                escape_sequence.push(c as u8);
            } else if c == '\r' { // If the user presses Enter
                println!();
                let trimmed_input = input.trim(); // Remove leading and trailing whitespace
                history.push(trimmed_input.to_string()); // Add the command to history

                if trimmed_input == "exit" {
                    disable_raw_mode().unwrap();
                    return Ok(());
                }

                let mut parts = trimmed_input.split_whitespace();
                if let Some(command) = parts.next() {
                    let args: Vec<&str> = parts.collect();

                    match command {
                        "cd" => {
                            let new_dir = args.first().map_or("/", |&dir| dir);
                            if let Err(e) = env::set_current_dir(new_dir) {
                                eprintln!("{}", e);
                            }
                        }
                        "history" => {
                            for entry in &history {
                                println!("{}", entry);
                            }
                        }
                        command => {
                            let child: Result<Child, io::Error> = Command::new(command)
                                .args(&args)
                                .spawn();

                            match child {
                                Ok(mut child) => { let _ = child.wait(); }
                                Err(e) => eprintln!("{}", e),
                            }
                        }
                    }
                }

                input.clear(); // Clear the input buffer for the next command
                break;
            } else if c == '\x08' || c == '\x7F' { // Handle backspace (ASCII codes for backspace)
                backspace(&mut input);
            } else if c.to_string() == "\x1B[A" {
                historyup(&mut input, path.clone(), &mut history, historyindex)
            } else {
                appendbuf(&mut input, c);
            }

            if c == '\x1A' { // If the user presses ctrl+z, disable raw mode
                disable_raw_mode().unwrap();
            }
        }
    }
}
