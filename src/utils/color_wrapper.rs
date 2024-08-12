// Parts of this code are based on https://github.com/Aloso/to-html/blob/main/LICENSE,
// which is MIT-licensed.

use lazy_static::lazy_static;
use regex::Regex;

lazy_static! {
    pub static ref ANSI_REGEX: Regex =
        Regex::new(r"\u{1b}(\[[0-9;?]*[A-HJKSTfhilmnsu]|\(B)").unwrap();
}

/// This function receives a string with color and highlighting information
/// in ANSI color code format and treats them to be reported by the language server.
///
/// As tracked in issue #1, diagnostic highlighting still does not implement color
/// information in VSCode, so for now we just remove color codes.
pub fn treat_colors(text: &str) -> String {
    ANSI_REGEX.replace_all(text, "").to_string()
}
