//! Parser module:
//! This module parses the file using the `Linearizator` trait, first
//! `linearizator()` matches the corresponding implementation of the
//! `Lineatizator` trait, then that trait using regex gets the matches for
//! comments, substracting that value with the actual lines of the source we 
//! get the lines of code, also we skip the dir who are used to contain 
//! external dependencies source code, or intermediate precompiled code,
//! ex: "node_modules" (js), "target" (rust), "out" (typescript, wasm, etc...)

use std::fs::File;
use std::io::Read;
use std::path::Path;

use lazy_static::{__Deref, lazy_static};
use regex::Regex;

/// Given an extension it returns the corresponding `Lineatizator` that will
/// parse the file
fn linearizator(extension: &str) -> Option<Box<dyn Linearizator>> {
    // Match the extension and return the desired Linearizator
    match extension {
        "rs" | "c" | "cpp" | "cxx" | "js" 
        | "ts" | "jsx" | "ejs" | "java" | "go"
                            => Some(Box::new(DefaultLinearizator {})),
        "py" | "pyc" | "pyx"
                            => Some(Box::new(PythonLinearizator {})),
        _                   => None,
    }
}

/// Abstract trait for all the future implementations of a Linearizator
/// (needed better name I know). It counts the lines of a file following this
/// rule: `lines` = `total_lines` - `lines_commented`
trait Linearizator {
    /// This trait method is shared among all the Linearizators and counts the
    /// number of lines of code making use of the `get_comments` which is 
    /// implementation especific
    fn count_lines(&self, input: String) -> usize {
        // Substracts the total lines of code inside the file and the lines of
        // all the comments
        input.lines().count()
            - self
                .get_comments()
                .find_iter(input.as_str())
                .map(|s| s.as_str().lines().count())
                .sum::<usize>()
    }

    /// Get a Regex for the specific `Linearizator` implementation that matches
    /// all kinds of comments
    fn get_comments(&self) -> Regex;
}

struct DefaultLinearizator {}
impl Linearizator for DefaultLinearizator {
    fn get_comments(&self) -> Regex {
        lazy_static! {
            static ref RE: Regex =
                Regex::new(r"(/\*[\w\s\n]*\*/)|(/+[\s\w]*)").unwrap();
        }
        RE.deref().clone()
    }
}

struct PythonLinearizator {}
impl Linearizator for PythonLinearizator {
    fn get_comments(&self) -> Regex {
        lazy_static! {
            static ref RE: Regex =
                Regex::new(r#"(#+[\s\w]*)|("""[\w\s\n]*""")"#).unwrap();
        }
        RE.deref().clone()
    }
}

/// Given an `input` and an `extension` (to match the kind of file to parse),
/// it returns the lines of code of that file, if not capable of parsing it
/// (not implemneted a parser yet) it counts 0
fn lines_of_code(input: String, extension: &str) -> usize {
    if let Some(ltor) = linearizator(extension) {
        return ltor.count_lines(input);
    } else {
        // println!("Unimplemented extension {}", extension);
        return 0;
    }
}

/// Given a `filepath` it will parse it and return the line count keeping in
/// mind that can have comments. Its not perfect and it does not aim to be,
/// just fast
pub fn parse_file(filepath: impl AsRef<Path>) -> Option<usize> {
    // Transforms the filepath arg to `Path`
    let filepath = filepath.as_ref();

    // read the contents of the file and process it
    let mut file_content: String = String::new();
    if let Ok(mut file) = File::open(filepath) {
        // if there are contents on the file it gets the lines of code
        if 0 < file.read_to_string(&mut file_content).ok()? {
            return Some(lines_of_code(
                file_content,
                filepath.extension()?.to_str()?,
            ));
        } else {
            return Some(0);
        }
    }
    None
}

/// Simple tests for each `Linearizator` implementation
#[cfg(test)]
mod parser {
    use super::*;
    #[test]
    fn default_test() {
        assert_eq!(
            lines_of_code(
                &"// aaa\n// aaa\n// aaa\n/*\naaa\naaa\naaa\n*/\nHi"
                    .to_string(),
                "rs"
            ),
            1
        );
    }

    #[test]
    fn python_test() {
        assert_eq!(
            lines_of_code(
                &r#"# Hello
"""" 
Multiline
Comment
""""
print("Hello")"#
                    .to_string(),
                "py"
            ),
            1
        );
    }

    #[test]
    fn it_works() {
        assert!(0 == 0);
    }
}
