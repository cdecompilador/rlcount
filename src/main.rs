use std::fmt;
use std::fs;
use std::io;
use std::path::Path;
use std::path::PathBuf;

use walkdir::WalkDir;
use rayon::prelude::*;
use clap::{Arg, App, SubCommand};

mod parser;
use parser::parse_file;

/// Return the language name given the `extension`
/// TODO: Add support for more languages
fn get_language_name<'a>(extension: &'a str) -> &'a str {
    match extension {
        "rs" => "Rust",
        "c" => "C",
        "cpp" | "cxx" | "c++" => "C++",
        "py" => "Python",
        "js" | "jsx" | "ejs" => "Javascript",
        "ts" | ".d.ts" => "Typescript",
        "html" => "HTML",
        "css" => "css",
        _ => "Unknown"
    }
}

/// Function that given a `dir` fills the vec `filenames` with a recursive
/// search in the project of extensioned files. Maybe not using recursion
/// could be a good option
fn get_files<P: AsRef<Path>>(
    dir: P,
    filenames: &mut Vec<PathBuf>,
) -> io::Result<()> {
    for entry in WalkDir::new(dir).into_iter().filter_map(|e| e.ok()) {
        filenames.push(entry.path().to_path_buf());
    }
    Ok(())
}

/// The representation of the data that will be yielded to the user by Debug
/// trait, designed with threading in mind. It should be contained into an Arc
/// and changes from multriple threads will push info via the `push` function
pub struct ProjectData {
    lines_per_language: Vec<(String, usize)>,
    total_lines: usize,
    name: String,
    percentage_per_language: Vec<(String, f64)>,
}

impl ProjectData {
    /// Creates a new projett data with a especified name
    pub fn new(name: &str) -> Self {
        ProjectData {
            // Ej: [("c", 129), ("rs", 48)]
            lines_per_language: Vec::with_capacity(8), // Arbitrary number
            // To be calculate when collapse is called
            total_lines: 0,
            name: name.to_owned(),
            // TODO: To be calculate when collapse is called
            percentage_per_language: Vec::with_capacity(8),
        }
    }

    /// Push the data obtained to the project data, designed to be threaded
    pub fn push(&mut self, lang_name: &str, lines: usize) -> Option<()> {
        // If the lang is already contained update it if not push the new one
        if self
            .lines_per_language
            .iter()
            .any(|(lang, _)| lang_name == lang)
        {
            let (_, l) = self
                .lines_per_language
                .iter_mut()
                .find(|(lang, _)| lang_name == lang)?; // Not failable because checked before
            *l += lines;
        } else {
            self.lines_per_language.push((lang_name.to_owned(), lines));
        }

        Some(())
    }

    /// Compile with the collected data the results
    pub fn collapse(&mut self) {
        self.total_lines = self
            .lines_per_language
            .iter()
            .fold(0, |acc, lines| acc + lines.1); // Fold it like an epic haskeller

        // Calculate the percentage of the total code of each language
        for (lang, lines) in &self.lines_per_language {
            let percentage = *lines as f64 / self.total_lines as f64 * 100.0;
            self.percentage_per_language
                .push((lang.clone(), percentage));
        }
    }
}

/// Print the project data with the objective to put in into the screen
impl fmt::Debug for ProjectData {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        /*
            Ex:
            PROJECT_NAME: test          TOTAL_LINES: 186

               RUST  => Lines: y      97.85 %
               C     => Lines: x      2.15  %
        */
        // Render into a string the percentages
        let mut percentages = String::from("\n");
        for (lang, perc) in &self.percentage_per_language {
            percentages.push_str(&format!(
                "    {:<14}=> Lines: {:<14?} {:>6.2} %\n",
                get_language_name(&lang),
                self.lines_per_language
                    .iter()
                    .find(|s| lang == &s.0)
                    .unwrap()
                    .1,
                perc
            ));
        }

        write!(
            f,
            "COMPLETE! {:<30}TOTAL_LINES: {}\n{}",
            self.name, self.total_lines, percentages
        )
    }
}

fn main() {
    let matches = App::new("Rust Line Counting")
        .version("0.2.0")
        .author("cdecompilador <nyagouno@gmail.com>")
        .about("Just that, it counts lines nwn")
        .arg(Arg::with_name("INPUT")
            .help("The dir where to start looking for source code")
            .required(true)
            .index(1))
        .get_matches();
    let mut filenames = Vec::new();
    get_files(matches.value_of("INPUT").unwrap(), &mut filenames)
        .expect("Error getting filenames");

    // TODO: Create the project_data with the name of the folder
    let mut project_data = ProjectData::new("");

    // Process each file in the project
    let a = filenames.par_iter()
        .map(|filename| {
        let extension: &str = match filename.extension() {
            Some(extension) => {
                let ext: &str = extension.to_str().unwrap();
                if get_language_name(ext) == "Unknown" {
                    return None;
                } 
                ext
            },
            None => {
                return None;
            }
        };
        if let Some(n) = parse_file(filename) {
            Some((extension, n))
        } else { None }
    }).collect::<Vec<Option<(&str, usize)>>>().iter().for_each(|val| {
        if let Some((extension, n)) = val {
            project_data.push(extension, *n);
        }
    });

    // Collapse the results and show them
    project_data.collapse();
    println!("{:?}", project_data);
}
