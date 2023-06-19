use std::error::Error;
use std::fmt;

#[derive(Clone, Debug)]
pub struct YamlProcessError {
    pub message: String,
}

impl Error for YamlProcessError {}

impl fmt::Display for YamlProcessError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Yaml Processing: {}", self.message)
    }
}
