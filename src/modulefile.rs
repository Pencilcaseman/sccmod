use crate::module::{Dependency, Environment, Module};

pub fn generate(module: &Module) -> String {
    // Generate a modulefile with support for flavours
    // The modulefile has the following format:

    let module_class = &module.class;

    let mut module_metadata_str = String::new();
    for (key, value) in module.metadata.iter() {
        module_metadata_str.push_str(&format!("# {key}: {value}\n"));
    }

    let mut module_metadata_str_no_hashes = String::new();
    for (key, value) in module.metadata.iter() {
        module_metadata_str_no_hashes.push_str(&format!("{key}: {value}\n"));
    }

    let no_description_provided = "No description provided".to_string();
    let module_description = module
        .metadata
        .get("description")
        .unwrap_or(&no_description_provided);

    let mut class_definitions = String::new();
    for class in module.dependencies.iter().filter_map(|dep| {
        if let Dependency::Class(name) = dep {
            Some(name)
        } else {
            None
        }
    }) {
        class_definitions.push_str(&format!("flavours prereq -class {class}\n"));
    }

    let root_dir = &module.install_path;

    let mut environment_variables = String::new();
    for (key, value) in module.environment.iter() {
        environment_variables.push_str(&match value {
            Environment::Set(val) => format!("setenv \"{key}\" \"{val}\"\n"),
            Environment::Append(val) => format!("flavours append-path \"{key}\" \"{val}\"\n"),
            Environment::Prepend(val) => format!("flavours prepend-path \"{key}\" \"{val}\"\n"),
        })
    }

    format!(
        r#"#%Module

# MODULEFILE GENERATED BY SCCMOD
# https://github.com/Pencilcaseman/sccmod

# Metadata
{module_metadata_str}

# Flavours initialisation
package require flavours
flavours init

# Module help
proc ModulesHelp {{ }} {{
   puts stderr "
{module_metadata_str_no_hashes}
"
}}

module-whatis "{module_description}"

# Module prerequisites
{class_definitions}

# Conflict with other modules of the same class
flavours conflict -class {module_class}

# Evaluate the flavour
flavours root     {root_dir}
flavours revision 1
flavours commit

# Set environment variables
{environment_variables}

# Cleanup and reload conflicting modules
flavours cleanup
"#
    )
}
