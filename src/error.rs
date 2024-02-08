
pub fn error(line: usize, message: String) {
    report(line, String::new(), message);
}

pub fn report(line: usize, location: String, message: String) {
    eprintln!("[line {line}] Error {location}: {message}");
}