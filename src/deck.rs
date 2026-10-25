use std::{fs::{self, File, metadata}, io::Result, os::unix::fs::MetadataExt, path::PathBuf};

struct Folder {
    dir: PathBuf,
    folders: Vec<Folder>,
    size: u64
}

impl Folder {
    fn new(dir: PathBuf) -> Self {
        Self {
            dir,
            folders: vec![],
            size: 0
        }
    }

    fn size_cal(&mut self) -> Result<u64> {
        let files = fs::read_dir(&self.dir)?;

        for file in files {
            let entry = file?;
            let md = entry.metadata()?;

            if md.is_file() {
                self.size += md.size();
            }
            else {
                let mut folder = Folder::new(entry.path());
                self.size += folder.size_cal()?;
                self.folders.push(folder);
            }
        }
        Ok(self.size)
    }
}

struct SpaceDeck {
    head: Folder
}

impl SpaceDeck {
    fn new(dir: PathBuf) -> Self {
        let head = Folder::new(dir);
        Self {
            head,
        }
    }

    fn size_cal(&mut self) {
        
    }
}

