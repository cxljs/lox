pub fn error(line: u32, msg: &str) {
    eprintln!("[line {}] Error: {}", line, msg);
}
