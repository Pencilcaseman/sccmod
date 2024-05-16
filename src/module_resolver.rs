use crate::{
    log,
    module::{get_modules, Module},
};

pub enum ResolveMatch {
    Full(Module),
    Partial(Vec<Module>),
    None,
}

/// Resolve a module identifier (or set of identifiers) from a set of
/// partial identifiers. Each of the provided partials must appear in
/// the module identifier.
///
/// # Errors
/// Will error if the modules cannot be listed.
pub fn resolve(partials: &[&str]) -> Result<ResolveMatch, String> {
    let mut results: Vec<Module> = get_modules()?
        .into_iter()
        .filter(|module| {
            partials.iter().map(|p| p.to_lowercase()).all(|partial| {
                module
                    .identifier
                    .to_lowercase()
                    .contains(&partial.to_lowercase())
            })
        })
        .collect();

    match results.len() {
        0 => Ok(ResolveMatch::None),
        1 => Ok(ResolveMatch::Full(results.pop().map_or_else(
            || log::error("An internal error has occurred"),
            |x| x,
        ))),
        _ => Ok(ResolveMatch::Partial(results)),
    }
}
