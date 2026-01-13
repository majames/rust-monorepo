pub fn report_error(line: u64, w: &str, message: &str) {
    println!("[line {line}] Error {w}: {message}");
}
