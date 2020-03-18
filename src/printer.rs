use reqwest::Url;
use std::fmt::Display;
use std::io::{stdout, Write};

pub struct Printer {
    show_progress: bool,
    verbosity: u64,
    cleaner: &'static str
}

impl Printer {

    const TERMINAL_CLEANER_STRING: &'static str =  "\r\x1b[2K";

    pub fn new(
        show_progress: bool,
        verbosity: u64,
    ) -> Self {

        let cleaner;
        if show_progress {
            cleaner = Self::TERMINAL_CLEANER_STRING;
        }
        else {
            cleaner = "";
        }

        return Self{
            show_progress,
            verbosity,
            cleaner
        };
    }

    pub fn print_url(&self, url: &Url) {
        println!("{}{}", self.cleaner, url);
    }

    pub fn print_error<E: Display>(&self, err: &E) {
        if self.verbosity > 0 {
            eprintln!("{}[-] {}: ", self.cleaner, err);
        }
    }

    pub fn print_progress(&self, current_count: usize, max_count: usize) {
        if self.show_progress {
            let percentage = current_count as f32 / max_count as f32 * 100.0;

            print!(
                "{}{}/{} {:.2}%",
                self.cleaner,
                current_count, 
                max_count, 
                percentage
            );
            let _ = stdout().flush();
        }
    }

    pub fn print_end(&self) {
        print!("{}", self.cleaner);
    }
}