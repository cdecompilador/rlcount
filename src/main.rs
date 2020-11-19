use std::fmt;
use std::fs;
use std::io;
use std::path::Path;
use std::path::PathBuf;

/// Return the language name given the `extension`
fn get_language_name<'a>(extension: &'a str) -> &'a str {
    match extension {
        "rs" => "Rust",
        "c" => "C",
        "cpp" | "cxx" | "c++" => "C++",
        "py" => "Python",
        "js" | "jsx" => "Javascript",
        "ts" => "Typescript",
        _ => {
            println!("Uknown file extension: {}", extension);
            "Uknown"
        },
         // TODO: Add more
    }
}
/// TODO: Add more languages
const KNOWN_EXTENSIONS_BINDINGS: &[(&str, &[&str])] = &[
    ("rs", &["//", "/*", "*/"]),
    ("c", &["//", "/*", "*/"]),
    ("cpp", &["//", "/*", "*/"]),("c++", &["//", "/*", "*/"]),("cxx", &["//", "/*", "*/"]),
    ("py", &["#", "\"\"\"", "\"\"\""]),
    ("js", &["//", "/*", "*/"]),("jsx", &["//", "/*", "*/"]),
    ("ts", &["//", "/*", "*/"]),
];

/// Representation of a Line
#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub enum Line {
    SingleComment,
    OpenMultiComment,
    CloseMultiComment,
    Normal,
}

/// Given a `filepath` it will parse it and return the line count keeping in mind
/// that can have comments. Its not perfect and it does not aim to be, just fast
fn parse_file<P: AsRef<Path>>(filepath: P) -> Option<usize> {
    let ext = filepath.as_ref().extension()?;
    if !KNOWN_EXTENSIONS_BINDINGS.iter().any(|x| x.0 == ext) {
        return None;
    }
    let mut single_line_match = None;
    let mut open_multiline_match = None;
    let mut close_multiline_match = None;
    for e in KNOWN_EXTENSIONS_BINDINGS {
        if e.0 == ext {
            single_line_match = Some(e.1[0]);
            open_multiline_match = Some(e.1[1]);
            close_multiline_match = Some(e.1[2]);
        }
    }
    let single_line_match = single_line_match?;
    let open_multiline_match = open_multiline_match?;
    let close_multiline_match = close_multiline_match?;
    // TODO: Best error reporting
    // Read the file given the `filepath`
    let file = match fs::read_to_string(filepath) {
        Ok(f) => f,
        Err(_) => return None,
    };
    let mut p_lines: Vec<Line> = Vec::with_capacity(file.len());
    // Iterate the lines and push the type of each one
    for l in file.lines() {
        let l = l.trim();
        if l.starts_with(single_line_match) {
            p_lines.push(Line::SingleComment);
        } else if l.starts_with(open_multiline_match) {
            p_lines.push(Line::OpenMultiComment);
        } else if l.contains(close_multiline_match) { // To avoid line count loss
            p_lines.push(Line::CloseMultiComment);
        } else {
            p_lines.push(Line::Normal);
        }
    }
    // Count the Line::Normal except the ones between a multiline comment
    let mut in_multi = false;
    let mut line_count = 0;
    for l in p_lines.iter() {
        match l {
            Line::OpenMultiComment => {
                in_multi = true;
            }
            Line::CloseMultiComment => {
                if in_multi {
                    in_multi = false;
                } else {
                    // Case of a close comment fount but no open
                    line_count += 1; 
                }
            }
            Line::SingleComment => {}
            Line::Normal => {
                if !in_multi {
                    line_count += 1;
                }
            }
        };
    }
    Some(line_count)
}

/// Function that given a `dir` fills the vec `filenames` with a recursive search
/// in the project of extensioned files
fn get_files<P: AsRef<Path>>(dir: P, filenames: &mut Vec<PathBuf>) -> io::Result<()> {
    // Get the entries in dir
    let entries = fs::read_dir(dir)?;

    for entry in entries {
        // If a entry is a file add it to the entries and if is not, recursive call to this func
        let entry = entry?;
        if entry.file_type()?.is_file() {
            filenames.push(entry.path());
        } else  if entry.file_type()?.is_dir() {
            get_files(entry.path(), filenames)?;
        } else {
            // TODO: Symlink encountered, don't know how to manage it
        }
    }
    // Filter non extensioned files
    filenames.retain(|f| f.extension().is_some());
    Ok(())
}

/// The representation of the data that will be yielded to the user by Debug trait,
/// designed with threading in mind. It should be contained into an Arc and changes from multriple
/// threads will push info via the `push` function
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
                "    {:<7}=> Lines: {:<7?} {:>6.2} %\n",
                get_language_name(&lang),
                self.lines_per_language.iter().find(|s| lang == &s.0).unwrap().1,
                perc
            ));
        }

        write!(
            f,
            "PROJECT_NAME: {:<30}TOTAL_LINES: {}\n{}",
            self.name, self.total_lines, percentages
        )
    }
}

fn main() {
    // Get the dir from args
    let path = &std::env::args().nth(1).expect("Usage: rlcount <path>");

    // Get all the filenames from the path
    let mut filenames = Vec::new();
    get_files(path, &mut filenames).expect("Error getting filenames");

    // TODO: Create the project_data with the name of the folder
    let mut project_data = ProjectData::new("...");

    // Process each file in the project
    for file in filenames.iter() {
        let extension = file.extension().unwrap().to_str().unwrap();
        if let Some(n) = parse_file(file) {
            project_data.push(extension, n);
        }
    }

    // Collapse the results and show them
    project_data.collapse();
    println!("{:?}", project_data);
}
