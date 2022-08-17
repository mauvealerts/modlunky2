use std::path::PathBuf;

use crate::{data::Version, Result};

pub struct Binary {
    ml2_path: PathBuf,
}

impl Binary {
    pub fn new(ml2_path: &str) -> Self {
        Self {
            ml2_path: ml2_path.into(),
        }
    }

    fn version_dir(&self, version: Version) -> Result<PathBuf> {
        let name = match version {
            Version::Local(path) => return Ok(path.into()),
            Version::Nightly => "nightly",
            Version::Stable => "stable",
        };
        let path = self.ml2_path.join("playlunky").join(name);
        Ok(path)
    }
}
