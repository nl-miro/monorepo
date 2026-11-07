fn main() {
    let result = std::thread::Builder::new()
        .stack_size(64 * 1024 * 1024)
        .spawn(development::valueless::run)
        .expect("spawn valueless worker")
        .join()
        .expect("valueless worker panicked");

    match result {
        Ok(true) => {}
        Ok(false) => std::process::exit(1),
        Err(error) => {
            eprintln!("error: {error}");
            std::process::exit(2);
        }
    }
}
