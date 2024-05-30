use crate::module::{get_modules, Dependency, Module};

pub fn generate(module: &Module) -> Result<Vec<(Vec<Module>, usize)>, String> {
    let modules = get_modules()?;

    // 1. Extract dependent modules and classes
    let required_modules: Vec<&Module> = module
        .dependencies
        .iter()
        .filter_map(|dep| {
            if let Dependency::Module(name) = dep {
                Some(name)
            } else {
                None
            }
        })
        .map(|name| {
            modules
                .iter()
                .find(|m| (&m.identifier() == name) || (&m.mod_name() == name))
                .ok_or(format!(
                    "Failed to find module matching dependency '{name}'"
                ))
        })
        .collect::<Result<Vec<&Module>, String>>()?;

    let required_classes: Vec<String> = module
        .dependencies
        .iter()
        .filter_map(|dep| {
            if let Dependency::Class(name) = dep {
                Some(name.to_owned())
            } else {
                None
            }
        })
        .collect();

    // 2. Extract modules with matching class
    let available_per_class: Vec<Vec<&Module>> = required_classes
        .iter()
        .map(|class| modules.iter().filter(|m| &m.class == class).collect())
        .collect();

    // 3. Generate permutations of the classes
    let mut index = vec![0usize; required_classes.len() + 1];

    let mut permutations = Vec::new();

    let end = required_classes.len();
    while index[end] == 0 {
        let mut perm: Vec<Module> = (0..end)
            .zip(index.iter().enumerate())
            .map(|(_, (class, &idx))| available_per_class[class][idx].to_owned())
            .collect();

        // Add pre-defined modules
        perm.extend(required_modules.iter().map(|&m| m.to_owned()));

        let permutation = (perm, required_classes.len());

        permutations.push(permutation);

        index[0] += 1;

        let mut i = 0;
        while i < end && index[i] >= available_per_class[i].len() {
            index[i] = 0;
            index[i + 1] += 1;
            i += 1;
        }
    }

    Ok(permutations)
}

pub fn gen_name(flav: &(Vec<Module>, usize)) -> String {
    let mut flav_str = "|".to_string();
    for i in 0..flav.1 {
        flav_str.push_str(&flav.0[i].mod_name());
        flav_str.push_str("|")
    }
    flav_str
}
