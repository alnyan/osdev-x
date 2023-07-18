pub const PARENT_NAME: &'static str = "..";
pub const SELF_NAME: &'static str = ".";

pub const SEPARATOR_STR: &'static str = "/";
pub const SEPARATOR: char = '/';

pub fn split_left(path: &str) -> (&str, &str) {
    if let Some((left, right)) = path.split_once(SEPARATOR) {
        (left, right.trim_start_matches(SEPARATOR))
    } else {
        (path, "")
    }
}

pub fn split_right(path: &str) -> (&str, &str) {
    if let Some((left, right)) = path.trim_end_matches(SEPARATOR).rsplit_once(SEPARATOR) {
        (left.trim_end_matches(SEPARATOR), right)
    } else {
        ("", path)
    }
}
