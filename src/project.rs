use std::io::BufWriter;

use camino::Utf8PathBuf;
use serde::{Deserialize, Serialize};

pub type ProjectFilePath = Utf8PathBuf;

#[derive(Default, Serialize, Deserialize, Debug)]
pub struct Project {
    last_path: Option<Utf8PathBuf>,
    game_dir: Utf8PathBuf,
}

impl Project {
    pub fn new(game_dir: Utf8PathBuf) -> Self {
        Self {
            game_dir,
            ..Default::default()
        }
    }

    pub fn from_file<P: AsRef<std::path::Path>>(path: P) -> anyhow::Result<Self> {
        let project_file = std::fs::read(&path)?;
        let project = serde_json::from_slice(&project_file)?;
        let path = path.as_ref().to_owned();
        let path = match Utf8PathBuf::from_path_buf(path) {
            Ok(path) => path,
            Err(path) => anyhow::bail!("Failed to create Utf8PathBuf from {:?}", path),
        };
        Ok(Self {
            last_path: Some(path),
            ..project
        })
    }

    pub fn save_file<P: AsRef<std::path::Path>>(&mut self, path: P) -> anyhow::Result<()> {
        let path = path.as_ref().to_owned();
        let path = match Utf8PathBuf::from_path_buf(path) {
            Ok(path) => path,
            Err(path) => anyhow::bail!("Failed to create Utf8PathBuf from {:?}", path),
        };
        let project_file = std::fs::File::create(&path)?;
        let buf_writer = BufWriter::new(project_file);
        serde_json::to_writer(buf_writer, self)?;
        self.last_path = Some(path);
        Ok(())
    }

    pub fn last_path(&self) -> Option<Utf8PathBuf> {
        self.last_path.clone()
    }

    pub fn game_dir(&self) -> Utf8PathBuf {
        self.game_dir.clone()
    }

    pub fn enumerate_files(&self) -> Vec<ProjectFilesEntry> {
        fn visit_dir(dir: &std::path::Path) -> anyhow::Result<Vec<ProjectFilesEntry>> {
            let mut result = Vec::new();
            let mut files = Vec::new();
            if dir.is_dir() {
                for entry in std::fs::read_dir(dir)? {
                    let entry = entry?;
                    let path = entry.path();
                    let utf8_path = match Utf8PathBuf::from_path_buf(path) {
                        Ok(path) => path,
                        Err(_) => {
                            anyhow::bail!("dir entry path was not valid utf8: {:?}", entry.path())
                        }
                    };

                    if utf8_path.is_dir() {
                        result.push(ProjectFilesEntry::Directory((
                            utf8_path,
                            visit_dir(&entry.path())?,
                        )));
                    } else {
                        files.push(ProjectFilesEntry::File(utf8_path));
                    }
                }
            }
            result.extend(files.into_iter());
            Ok(result)
        }

        match visit_dir(self.game_dir().as_path().as_std_path()) {
            Ok(entries) => entries,
            Err(_) => Vec::new(),
        }
    }
}

#[derive(Debug)]
pub enum ProjectFilesEntry {
    File(Utf8PathBuf),
    Directory((Utf8PathBuf, Vec<ProjectFilesEntry>)),
}
