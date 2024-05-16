/// Read a colon-separated environment variable and return a vector of strings
///
/// If only one string is present, it is returned as a single-element vector.
///
/// # Errors
/// If the variable is not set or is invalid, [`Err`] is returned with an error message.
pub fn read_var(name: &str) -> Option<Vec<String>> {
    Some(
        std::env::var(name)
            .ok()?
            .split(':')
            .map(std::string::ToString::to_string)
            .collect(),
    )
}
