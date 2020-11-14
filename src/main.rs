use std::fs;
use std::io;
use std::path::Path;
use std::path::PathBuf;

const KNOWN_EXTENSIONS_BINDINGS: &[(&str, &[&str])] = &[
    ("rs", &["//", "/*", "*/"]),
    ("c", &["//", "/*", "*/"]),
    ("cpp", &["//", "/*", "*/"]),
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
/// that can have comments
fn parse_file<P: AsRef<Path>>(filepath: P) -> Option<usize> {
    let ext = filepath.as_ref().extension()?;
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
    // FIXME: There must be a better way to do this
    let single_line_match = match single_line_match {
        Some(s) => s,
        None => return None,
    };
    let open_multiline_match = match open_multiline_match {
        Some(s) => s,
        None => return None,
    };
    let close_multiline_match = match close_multiline_match {
        Some(s) => s,
        None => return None,
    };
    // TODO: Best error reporting
    // Read the file given the `filepath`
    let file = fs::read_to_string(filepath).expect("Error opening file");
    let mut p_lines: Vec<Line> = Vec::with_capacity(file.len());
    // Iterate the lines and push the type of each one
    for l in file.lines() {
        let l = l.trim();
        if l.starts_with(single_line_match) {
            p_lines.push(Line::SingleComment);
        } else if l.starts_with(open_multiline_match) {
            p_lines.push(Line::OpenMultiComment);
        } else if l.starts_with(close_multiline_match) || l.ends_with(close_multiline_match) {
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
                in_multi = false;
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
        // If a entry is a file add it and if is not recursive call to this func
        let entry = entry?;
        if entry.file_type()?.is_file() {
            filenames.push(entry.path());
        } else {
            get_files(entry.path(), filenames)?;
        }
    }
    // Filter non extensioned files
    filenames.retain(|f| f.extension().is_some());
    Ok(())
}

fn main() {
    // Get the dir from args
    let path = std::env::args().nth(1).expect("Usage: rlcount <path>");
    let mut filenames = Vec::new();
    get_files(path, &mut filenames).expect("Error getting filenames");
    let mut total_count = 0;
    for file in filenames.iter() {
        total_count += match parse_file(file) {
            Some(n) => {
                println!("Parsing lines of: {:?}, which yields: {}", file, n);
                n
            }
            None => {
                println!("Uknown extension: {:?}", file.extension());
                0
            }
        };
    }

    println!("Total count: {}", total_count);
}
