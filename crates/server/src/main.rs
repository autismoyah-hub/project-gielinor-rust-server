#[cfg(windows)]
mod application;
fn main() {
    #[cfg(windows)]
    if let Err(error) = application::run() {
        eprintln!("Server: {error}");
        std::process::exit(1);
    }
    #[cfg(not(windows))]
    eprintln!("This development executable currently requires Windows account storage.");
}
