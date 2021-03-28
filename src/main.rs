//! Main module that is responsible of the interactions with the user,
//! TODO: Move some of the functions of this file to another modules
use std::fmt;
use std::io;
use std::path::{Path, PathBuf};

use clap::{App, Arg};
use rayon::prelude::*;
use walkdir::{DirEntry, WalkDir};

mod parser;
use parser::parse_file;

/// Return the language name given the `extension`
/// TODO: Add support for more languages
fn get_language_name<'a>(extension: &'a str) -> &'a str {
    // Match the extension and return the name of the language for logging to 
    // the user mainly
    match extension {
        "rs"                    => "Rust",
        "c"                     => "C",
        "cpp" | "cxx" | "c++"   => "C++",
        "py"                    => "Python",
        "js" | "jsx" | "ejs"    => "Javascript",
        "ts"                    => "Typescript",
        "html"                  => "HTML", // Not yet
        "css"                   => "css",   // Not yet
        "java"                  => "Java",
        "go"                    => "Golang",
        _                       => "Unknown",
    }
}

/// Check if the dir is valid, if not do not look for files inside it, avoid
/// special project dirs like `node_modules` that contain source but not from
/// the especific project to analyse
fn is_invalid(entry: &DirEntry) -> bool {
    entry.file_name().to_str()
        .map(|s| {
            (s.starts_with(".") && s != ".") 
                || s == "target"
                || s == "node_modules"
                || s == "build"
                || s == "out"
        }).unwrap_or(false)
}

/// Function that given a `dir` fills the vec `filenames` with a recursive
/// search in the project of extensioned files. Maybe not using recursion
/// could be a good option
fn get_files<P: AsRef<Path>>(dir: P, filenames: &mut Vec<PathBuf>) 
    -> io::Result<()> {
    // Instantiate a walker for the dir that will allow iterate over all its
    // recursive childs
    let walker = WalkDir::new(dir).into_iter();

    // Iterate over all its recursive childs and add them to the filenames 
    // vector if they are valid
    for entry in walker.filter_entry(|e| !is_invalid(e)) {
        filenames.push(entry?.path().to_path_buf());
    }

    Ok(())
}

/// The representation of the data that will be yielded to the user by Debug
/// trait, designed with threading in mind. It should be contained into an Arc
/// and changes from multriple threads will push info via the `push` function
/// NOTE: Not yet
pub struct ProjectData {
    /// Tuples of (LaguageName, LinesOfCode)
    lines_per_language: Vec<(String, usize)>,

    /// Total lines of the project, set to 0 till `collapse` is called
    total_lines: usize,

    /// the name of the project, for the moment not supported
    name: String,

    /// Percentage of lines per language
    percentage_per_language: Vec<(String, f64)>,
}

impl ProjectData {
    /// Creates a new projett data with a especified name
    pub fn new(name: &str) -> Self {
        ProjectData {
            lines_per_language: Vec::with_capacity(8), 
            total_lines: 0,
            name: name.to_owned(),
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

    /// Collapse the data of the project to the fields uninitialized
    pub fn collapse(&mut self) {
        // Collapse the total lines of code including all the recognized 
        // languages, folding it like an epic haskeller
        self.total_lines = self.lines_per_language.iter()
            .fold(0, |acc, lines| acc + lines.1); 

        // Calculate the percentage of the total code of each language
        for (lang, lines) in &self.lines_per_language {
            let percentage = *lines as f64 / self.total_lines as f64 * 100.0;
            self.percentage_per_language.push((lang.clone(), percentage));
        }
    }
}

/// Print the project data with the objective to put in into the screen
/// with this format for example:
///```
///    Ex:
///    PROJECT_NAME: ...           TOTAL_LINES: 186
///
///       RUST  => Lines: y      97.85 %
///       C     => Lines: x      2.15  %
///```
impl fmt::Debug for ProjectData {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
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
    // Retrieve the arguments provided from the cli
    let matches = App::new("Rust Line Counting")
        .version("0.2.0")
        .author("cdecompilador <nyagouno@gmail.com>")
        .about("Just that, it counts lines nwn")
        .arg(
            Arg::with_name("INPUT")
                .help("The dir where to start looking for source code")
                .required(true)
                .index(1),
        ).get_matches();
    
    // Retireve the filenames
    let mut filenames = Vec::new();
    get_files(matches.value_of("INPUT").unwrap(), &mut filenames)
        .expect("Error getting filenames");
    
    // Create the `ProjectData` of the project
    let mut project_data = ProjectData::new("");

    // Process each file in the project, rayon is being used for the sake of
    // speed
    filenames.par_iter()
        .map(|filename| {
            // Extract the extension of the filename
            let extension: &str = match filename.extension() {
                Some(extension) => {
                    let ext: &str = extension.to_str().unwrap();
                    // If not supported the extension do nothing with it
                    if get_language_name(ext) == "Unknown" {
                        return None;
                    }
                    ext
                }
                None => {
                    return None;
                }
            };

            // Try to parse it and if possible save the extension with the 
            // retrieved lines of code
            if let Some(n) = parse_file(filename) {
                Some((extension, n))
            } else {
                None
            }
        })
        .collect::<Vec<Option<(&str, usize)>>>()
        .iter()
        // Once all the extensions with its lines of code are collected add 
        // them to the project data
        .for_each(|val| {
            // Used the `if let` to filter the Nones (the non-compatible files)
            if let Some((extension, n)) = val {
                project_data.push(extension, *n);
            }
        });

    // Collapse the results and show them
    project_data.collapse();
    println!("{:?}", project_data);
}
