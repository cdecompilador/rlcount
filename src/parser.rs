use std::path::Path;
use std::fs::{self, File};
use std::io::Read;

use regex::Regex;

/// Used to know the kind of comments used in the especified language
/// TODO: Add more languages
const KNOWN_EXTENSIONS_BINDINGS: &[(&str, &[&str])] = &[
    ("rs", &["//", "/*", "*/"]),
    ("c", &["//", "/*", "*/"]),
    ("cpp", &["//", "/*", "*/"]),
    ("c++", &["//", "/*", "*/"]),
    ("cxx", &["//", "/*", "*/"]),
    ("py", &["#", "\"\"\"", "\"\"\""]),
    ("js", &["//", "/*", "*/"]),
    ("jsx", &["//", "/*", "*/"]),
    ("ts", &["//", "/*", "*/"]),
];

fn linearizator(extension: &str) -> Option<impl Linearizator> {
    match extension {
        "rs" | "c" | "cpp" | "cxx" | "js" | "ts" | "jsx" | "ejs"
            => Some(DefaultLinearizator {}),
        _ => None
    }
}

trait Linearizator {
    fn count_lines(self, input: &String) -> usize where Self: Sized {
        // Substracts the total lines of code inside the file and the lines of
        // all the comments
        input.lines().count() 
            - self.get_comments()
                .find_iter(input)
                .map(|s| s.as_str().lines().count()).sum::<usize>()
    }

    fn get_comments(self) -> Regex where Self: Sized;
}

struct DefaultLinearizator {}
impl Linearizator for DefaultLinearizator {
    fn get_comments(self) -> Regex where Self: Sized {
        Regex::new(r"(/\*[\w\s\n]*\*/)|(/+[\s\w]*)").unwrap()
    }
}

fn lines_of_code(input: &String, extension: &str) -> usize {
    if let Some(ltor) = linearizator(extension) {
        return ltor.count_lines(input);
    } else {
        println!("Unimplemented extension {}", extension);
        return 0;
    }
}

/// Given a `filepath` it will parse it and return the line count keeping in
/// mind that can have comments. Its not perfect and it does not aim to be,
/// just fast
pub fn parse_file(filepath: impl AsRef<Path>) -> Option<usize> {
    let filepath = filepath.as_ref();
    // read the contents of the file
    let mut file_content: String = String::new();
    if let Ok(mut file) = File::open(filepath) {
        // if there are contents on the file it gets the lines of code
        if 0 < file.read_to_string(&mut file_content).ok()? {
            return Some(
                lines_of_code(
                    &file_content,
                    filepath.extension()?.to_str()?
                )
            );
        } else {
            return Some(0);
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn rust_tests() {
        assert_eq!(
            lines_of_code(&"// aaa\n// aaa\n// aaa\n/*\naaa\naaa\naaa\n*/\nHi".to_string(),"rs"),
            1
        );
    }

    #[test]
    fn it_works() {
        assert!(0 == 0);
    }
}

