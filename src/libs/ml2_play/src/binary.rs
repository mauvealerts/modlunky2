use std::path::{Path, PathBuf};

use tokio::sync::mpsc;

use crate::{
    data::{Event, Version},
    Result,
};

const NIGHTLY_SUBPATH: &str = "nightly";
const PLAY_SUBPATH: &str = "playlunky";
const STABLE_SUBPATH: &str = "stable";

const EXE_NAME: &str = "playlunky.exe";
const ARG_PREFIX: &str = "--exe_dir=";

pub struct Binary {
    install_dir: String,
    play_dir: PathBuf,
}

impl Binary {
    pub fn new(install_dir: &str, ml2_path: impl AsRef<Path>, ver: Version) -> Self {
        let play_dir = match ver {
            Version::Local(path) => path.into(),
            Version::Nightly => ml2_play_dir(ml2_path, NIGHTLY_SUBPATH),
            Version::Nightly => ml2_play_dir(ml2_path, STABLE_SUBPATH),
        };
        Self {
            install_dir: install_dir.into(),
            play_dir,
        }
    }

    pub async fn launch(&self, ver: Version, events_tx: mpsc::Sender<Event>) -> Result<()> {
        let path = self.play_dir.join(EXE_NAME);
        let arg = ARG_PREFIX.to_string().push_str(&self.install_dir);
        todo!()
    }
}

fn ml2_play_dir(ml2_path: impl AsRef<Path>, name: &str) -> PathBuf {
    ml2_path.as_ref().join(PLAY_SUBPATH).join(name)
}
