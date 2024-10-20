use crate::{
    log,
    module::{get_modules, Module},
};

pub enum ResolveMatch {
    Full(Module),
    Partial(Vec<Module>),
    All(Vec<Module>),
    None,
}

/// Resolve a module identifier (or set of identifiers) from a set of
/// partial identifiers. Each of the provided partials must appear in
/// the module identifier.
///
/// # Errors
/// Will error if the modules cannot be listed.
pub fn resolve(partials: &[&str]) -> Result<ResolveMatch, String> {
    // Allow passing `ALL` as the last argument to install all matching
    // modules

    let mut partials = partials.to_vec();
    let mut all = false;
    if partials[partials.len() - 1] == "ALL" {
        // Remove `--all` from the list of partials
        partials.pop();
        all = true;
    }

    let mut results: Vec<Module> = get_modules()?
        .into_iter()
        .filter(|module| {
            partials.iter().map(|p| p.to_lowercase()).all(|partial| {
                module
                    .identifier()
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
        _ if !all => Ok(ResolveMatch::Partial(results)),
        _ => Ok(ResolveMatch::All(results)),
    }
}
