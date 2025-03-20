
fn main() {
    // Generates built file for aquiring built info in programm
    built::write_built_file().expect("Failed to acquire build-time information");
}
