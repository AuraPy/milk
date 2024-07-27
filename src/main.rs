use std::process::{Command, Child};
use std::io::{self, stdin, stdout, Write, Read};
use std::env;
use crossterm::{terminal::{enable_raw_mode, disable_raw_mode}, cursor};

fn main() -> std::io::Result<()> {
    let mut history: Vec<String> = Vec::new();
    enable_raw_mode().unwrap(); // Enable raw mode for capturing input
    let mut input = String::new(); // Input buffer

    loop {
        let path = env::current_dir()?;
        print!("\r{}> ", path.display());
        stdout().flush().unwrap(); // Ensure prompt is displayed

        for b in stdin().bytes() {
            let c = b.unwrap() as char;

            if c == '\r' { // If the user presses Enter
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
                if !input.is_empty() {
                    input.pop(); // Remove the last character from the input buffer
                    print!("{} {}", cursor::MoveLeft(1), cursor::MoveLeft(1)); // Move cursor back and clear character
                    stdout().flush().unwrap();
                }
            } else {
                input.push(c); // Add the character to the input buffer
                print!("{}", c);
                stdout().flush().unwrap(); // Display the character as it's typed
            }

            if c == 'z' { // If the user presses 'z', disable raw mode
                disable_raw_mode().unwrap();
                return Ok(());
            }
        }
    }
}
