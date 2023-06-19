mod util;
mod yaml_process_error;

use crate::logger;
use linked_hash_map::LinkedHashMap;
use std::collections::HashMap;
use std::error::Error;
use std::path::Path;
use util::ReplaceResult;
use yaml_rust::{Yaml, YamlEmitter};

pub fn process(
    input_file_path: &str,
    base_path: &Path,
    log: &logger::Logger,
) -> Result<String, Box<dyn Error>> {
    log.println(true, "Loading the input file");
    let yaml = util::load_file(Path::new(input_file_path))?;
    log.println(true, "Start to process includings...");
    let included = including(&yaml, &base_path, &log)?;
    log.println(true, "Processing includings done");
    let applied = apply_generic(&included, &log);
    log.println(true, "Making the yaml string");
    let result = make_yaml_to_doc(&applied, log);
    return Ok(result);
}

fn including(
    yaml: &Yaml,
    base_path: &Path,
    log: &logger::Logger,
) -> Result<Yaml, Box<dyn Error>> {
    let including_key = Yaml::from_str("$include");
    let fn_including = |_: Option<&Yaml>, value: &Yaml| -> ReplaceResult {
        match value {
            Yaml::Hash(hash) => {
                if hash.contains_key(&including_key) {
                    let mut result = LinkedHashMap::new();
                    let target_path =
                        hash.get(&including_key).unwrap().as_str().unwrap();
                    let include_path = get_include_path(base_path, target_path);
                    let including_items =
                        util::load_several_files(include_path.as_str(), log);
                    for item in including_items {
                        util::concat_hash(&mut result, &item);
                    }
                    ReplaceResult::Replace(Yaml::Hash(result))
                } else {
                    ReplaceResult::NoReplace
                }
            }
            _ => ReplaceResult::NoReplace,
        }
    };

    let result_yaml = util::map(yaml, &fn_including);
    match result_yaml {
        Some(result_value) => Ok(result_value),
        None => Err(yaml_process_error::YamlProcessError {
            message: format!("An error occurs while processing the includings"),
        }
        .into()),
    }
}

fn get_include_path(base_path: &Path, target_path: &str) -> String {
    let target_path = Path::new(target_path);
    let include_path;
    if target_path.is_relative() {
        include_path = base_path.join(target_path);
    } else {
        include_path = Path::new(target_path).to_path_buf();
    }
    return String::from(include_path.to_str().unwrap());
}

fn apply_generic(yaml: &Yaml, log: &logger::Logger) -> Yaml {
    log.println(true, "\nSearch generic definitions...");
    let generics = find_generics(yaml, log);
    log.println(true, format!("found: {}", generics.len()).as_str());
    log.print(true, "\nProcessing generics...");
    let result = process_generics(yaml, &generics, log);
    log.println(true, "Done");
    return result;
}

fn process_generics(
    yaml: &Yaml,
    generics: &HashMap<String, Yaml>,
    log: &logger::Logger,
) -> Yaml {
    let result;
    match yaml {
        Yaml::Array(arr) => {
            let mut vec = Vec::new();
            for item in arr {
                vec.push(process_generics(&item, generics, log));
            }
            result = Yaml::Array(vec);
        }
        Yaml::Hash(hash) => {
            if hash.contains_key(&Yaml::from_str("$generic")) {
                let consumer = hash
                    .get(&Yaml::from_str("$generic"))
                    .expect("$generic must an object type")
                    .as_hash()
                    .unwrap();
                let generic_target_key = consumer
                    .get(&Yaml::from_str("target"))
                    .expect("$generic must have a target property")
                    .as_str()
                    .expect("$generic's target property must be a string");
                let generic_definition =
                    generics.get(generic_target_key).expect(
                        format!(
                            "There is no generic found for {}",
                            generic_target_key
                        )
                        .as_str(),
                    );
                result = make_generic_to_type(generic_definition, consumer);
            } else {
                let mut map = LinkedHashMap::new();
                for (key, item) in hash {
                    if yaml_to_string(key).ends_with("<GENERIC>") {
                        continue;
                    }
                    map.insert(
                        key.clone(),
                        process_generics(&item, generics, log),
                    );
                }
                result = Yaml::Hash(map);
            }
        }
        _ => {
            result = yaml.clone();
        }
    }
    return result;
}

fn make_generic_to_type(
    generic_definition: &Yaml,
    consumer: &LinkedHashMap<Yaml, Yaml>,
) -> Yaml {
    let (_, types) = extract_generic_types(consumer);
    return replace_generic(generic_definition, &types);
}

fn replace_generic(yaml: &Yaml, types: &HashMap<String, Yaml>) -> Yaml {
    let result: Yaml;

    match yaml {
        Yaml::Hash(hash) => {
            let mut map = LinkedHashMap::new();
            for (hkey, hval) in hash {
                match hval {
                    Yaml::String(val) => {
                        if types.contains_key(val) {
                            let target_type = types.get(val).unwrap();
                            map.insert(hkey.clone(), target_type.clone());
                        }
                    }
                    _ => {
                        map.insert(hkey.clone(), replace_generic(hval, types));
                    }
                }
            }
            result = Yaml::Hash(map);
        }
        Yaml::Array(arr) => {
            let mut vec = vec![];
            for item in arr {
                vec.push(replace_generic(&item, types));
            }
            result = Yaml::Array(vec);
        }
        _ => {
            result = yaml.clone();
        }
    }

    return result;
}

fn extract_generic_types(
    consumer: &LinkedHashMap<Yaml, Yaml>,
) -> (&str, HashMap<String, Yaml>) {
    let mut generic_name = "";
    let mut types = HashMap::new();
    for (original_key, t) in consumer {
        let key = yaml_to_string(original_key);
        if key == "target" {
            generic_name =
                t.as_str().expect("generic type must be a string value");
            continue;
        }
        types.insert(key, t.clone());
    }
    return (generic_name, types);
}

fn find_generics(yaml: &Yaml, log: &logger::Logger) -> HashMap<String, Yaml> {
    let mut result = HashMap::new();

    match yaml {
        Yaml::Array(ref v) => {
            for item in v {
                let map = find_generics(item, log);
                for (key, entry) in map {
                    result.insert(key, entry);
                }
            }
        }
        Yaml::Hash(ref v) => {
            for (original_key, item) in v.iter() {
                let key = yaml_to_string(original_key);
                if key.ends_with("<GENERIC>") {
                    let l = key.len();
                    let generic_key = key.chars().take(l - 9).collect();
                    log.println(true, format!("- {}", generic_key).as_str());
                    result.insert(generic_key, item.clone());
                } else {
                    let map = find_generics(item, log);
                    for (mkey, entry) in map {
                        result.insert(mkey, entry);
                    }
                }
            }
        }
        _ => (),
    }
    return result;
}

fn yaml_to_string(yaml: &Yaml) -> String {
    return match yaml {
        Yaml::Real(v) => String::from(v),
        Yaml::Integer(v) => v.to_string(),
        Yaml::String(v) => String::from(v),
        Yaml::Boolean(v) => v.to_string(),
        Yaml::Array(_) => String::from("Array"),
        Yaml::Hash(_) => String::from("Object"),
        Yaml::Null => String::from("Null"),
        _ => String::from("N/A"),
    };
}

fn make_yaml_to_doc(yaml: &Yaml, log: &logger::Logger) -> String {
    let mut out_str = String::new();
    let mut emitter = YamlEmitter::new(&mut out_str);
    emitter.dump(yaml).unwrap();
    log.println(
        true,
        format!("Output length: {} bytes", out_str.len()).as_str(),
    );
    return out_str;
}
