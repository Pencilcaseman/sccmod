use colored::Colorize;
use std::io::Write;

fn remove_tabs(txt: &str) -> String {
    txt.replace('\t', " ")
}

/// Print `message` as an error to the console and panic
#[allow(clippy::missing_panics_doc)]
pub fn error(message: &str) -> ! {
    println!(
        "{} : {}",
        "SCCMod Err".bold().truecolor(255, 0, 0),
        remove_tabs(message).italic().truecolor(255, 100, 25)
    );
    panic!("An error occurred");
}

/// Print `message` as a warning to the console
pub fn warn(message: &str) {
    println!(
        "{}: {}",
        "SCCMod Warn".bold().truecolor(255, 255, 0),
        remove_tabs(message).italic().truecolor(225, 225, 50)
    );
}

/// Print `message` to the console, marked as "informative"
pub fn info(message: &str) {
    println!(
        "{}: {}",
        "SCCMod Info".bold().truecolor(50, 150, 255),
        remove_tabs(message).italic().truecolor(50, 150, 255)
    );
}

/// Print `message` to the console, marked as "status"
pub fn status(message: &str) {
    println!(
        "{}: {}",
        "SCCMod Status".bold().truecolor(200, 65, 215),
        remove_tabs(message).italic().truecolor(230, 55, 235)
    );
}

/// Print `message` to the console, marked as `information`, but
/// append a carriage return so the cursor will print from the
/// start of the line on the next output.
///
/// # Panics
///
/// Panics if the call to [`std::io::stdout().flush()`] fails.
pub fn info_carriage(message: &str) {
    // Clear the line
    print!("\x1b[K");

    print!(
        "{}: {}\r",
        "SCCMod Info".bold().truecolor(50, 150, 255),
        remove_tabs(message).italic().truecolor(50, 150, 255)
    );

    std::io::stdout().flush().unwrap();
}

/// Print `message` to the console, marked as `warn`, but
/// append a carriage return so the cursor will print from the
/// start of the line on the next output.
///
/// # Panics
///
/// Panics if the call to [`std::io::stdout().flush()`] fails.
pub fn warn_carriage(message: &str) {
    // Clear the line
    print!("\x1b[K");

    print!(
        "{}: {}\r",
        "SCCMod Warn".bold().truecolor(255, 255, 0),
        remove_tabs(message).italic().truecolor(225, 225, 50)
    );

    std::io::stdout().flush().unwrap();
}
