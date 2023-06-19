use super::logger;
use super::util;
use super::yaml_process_error::YamlProcessError;
use glob::glob;
use linked_hash_map::LinkedHashMap;
use std::error::Error;
use std::path::Path;
use yaml_rust::{Yaml, YamlLoader};

pub fn load_file(path: &Path) -> Result<Yaml, Box<dyn Error>> {
    let input_content = std::fs::read_to_string(path);
    match input_content {
        Err(e) => Err(YamlProcessError {
            message: format!("Load '{}' error: {}", path.to_str().unwrap(), e),
        }
        .into()),
        Ok(v) => load_str(&v),
    }
}

pub fn load_several_files(
    include_path: &str,
    log: &logger::Logger,
) -> Vec<Yaml> {
    let mut result = vec![];

    let files = glob(include_path).unwrap();
    for entry in files {
        log.print(
            true,
            format!("[{}]: ", entry.as_ref().unwrap().display()).as_str(),
        );
        let inner_item = util::load_file(entry.as_ref().unwrap()).expect(
            format!("Load file [{}] failed", entry.unwrap().to_str().unwrap())
                .as_str(),
        );
        result.push(inner_item);
        log.println(true, "OK");
    }
    return result;
}

pub fn load_str(content: &str) -> Result<Yaml, Box<dyn std::error::Error>> {
    let docs = YamlLoader::load_from_str(content)?;
    if docs.len() < 1 || None == docs.get(0) {
        return Err(YamlProcessError {
            message: String::from("Input file contains no yaml entity"),
        }
        .into());
    }
    let doc = docs[0].clone();
    return Ok(doc);
}

#[allow(unused)]
pub enum TraverseDescent {
    Stop,
    Continue,
}

#[allow(unused)]
pub fn traverse<F>(yaml: &Yaml, callback: &F)
where
    F: Fn(Option<&Yaml>, &Yaml) -> TraverseDescent,
{
    match yaml {
        Yaml::Array(arr) => {
            for item in arr {
                match callback(None, item) {
                    TraverseDescent::Continue => {
                        traverse(item, callback);
                    }
                    _ => (),
                }
            }
        }
        Yaml::Hash(map) => {
            for (key, item) in map {
                match callback(Some(key), item) {
                    TraverseDescent::Continue => {
                        traverse(item, callback);
                    }
                    _ => (),
                }
            }
        }
        _ => {
            callback(None, yaml);
        }
    }
}

#[allow(unused)]
pub enum ReplaceResult {
    NoReplace,
    Delete,
    Replace(Yaml),
}

pub fn map<F>(yaml: &Yaml, callback: &F) -> Option<Yaml>
where
    F: Fn(Option<&Yaml>, &Yaml) -> ReplaceResult,
{
    match yaml {
        Yaml::Hash(hash) => {
            let mut result = LinkedHashMap::new();
            for (key, item) in hash {
                let replace_result = callback(Some(key), item);
                match replace_result {
                    ReplaceResult::Replace(replacement) => {
                        result.insert(key.clone(), replacement);
                    }
                    ReplaceResult::NoReplace => {
                        let map_result = map(&item, callback);
                        match map_result {
                            Some(replacement) => {
                                result.insert(key.clone(), replacement);
                            }
                            None => (),
                        }
                    }
                    _ => (),
                }
            }
            return Some(Yaml::Hash(result));
        }
        Yaml::Array(arr) => {
            let mut result = vec![];
            for item in arr {
                let replace_result = callback(None, item);
                match replace_result {
                    ReplaceResult::NoReplace => {
                        let map_result = map(&item, callback);
                        match map_result {
                            Some(replacement) => {
                                result.push(replacement);
                            }
                            None => (),
                        }
                    }
                    ReplaceResult::Replace(replacement) => {
                        result.push(replacement);
                    }
                    _ => (),
                }
            }
            return Some(Yaml::Array(result));
        }
        other => {
            let replace_result = callback(None, other);
            match replace_result {
                ReplaceResult::NoReplace => {
                    return Some(other.clone());
                }
                ReplaceResult::Replace(replacement) => {
                    return Some(replacement);
                }
                _ => None,
            }
        }
    }
}

pub fn concat_hash(hash: &mut LinkedHashMap<Yaml, Yaml>, yaml: &Yaml) {
    match yaml {
        Yaml::Hash(yaml_hash) => {
            for (ikey, ival) in yaml_hash {
                hash.insert(ikey.clone(), ival.clone());
            }
        }
        Yaml::Array(arr) => {
            for item in arr {
                concat_hash(hash, item);
            }
        }
        _ => (),
    }
}
