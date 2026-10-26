use std::{fs::{self, DirEntry, File, metadata}, io::Result, os::unix::fs::MetadataExt, path::PathBuf};

#[derive(Default)]
struct Folder {
    dir: PathBuf,
    folders: Vec<Folder>,
    files: Vec<PathBuf>,
    size: u64
}

impl Folder {
    fn new(dir: PathBuf) -> Self {
        Self {
            dir,
            folders: vec![],
            files: vec![],
            size: 0
        }
    }

    fn size_cal(&mut self) -> Result<u64> {
        let files = fs::read_dir(&self.dir);

        match files {
            Ok(files) => {
                for file in files {
                let entry = file?;
                let md = entry.metadata()?;

                if md.is_file() {
                    self.size += md.size();
                    self.files.push(entry.path());
                }
                else if md.is_dir() {
                    let mut folder = Folder::new(entry.path());
                    self.size += folder.size_cal()?;
                    self.folders.push(folder);
                }
        }
            },
            Err(e) => {
                eprintln!("Error: Could not read dir: {} {}", e,  self.dir.display().to_string())
            }
        }

        Ok(self.size)
    }

    fn print(mut self, depth: usize, limit: usize) -> Result<()>{

        if limit < depth {
            return Ok(());
        }

        if let Some(file) = self.dir.file_name() {
            let d = "-".repeat(depth) + &file.to_string_lossy() + ": " + &self.size.to_string();
            println!("{d}");
        }

        // for entry in self.files {
        //     let name = entry.file_name().to_string_lossy().into_owned();
        //     let output = "-".repeat(depth+1) + " " + &name;
        //     println!("{output}");
        // }
        

        self.folders.sort_by(|a, b| b.size.cmp(&a.size));

        for folder in self.folders {
            folder.print(depth+1, limit);
        }
        Ok(())
    }
}

#[derive(Default)]
pub struct SpaceDeck {
    head: Folder
}

impl SpaceDeck {
    pub fn new(dir: PathBuf) -> Self {
        let head = Folder::new(dir);
        Self {
            head,
        }
    }

    pub fn size_cal(&mut self) -> Result<u64> {
        self.head.size_cal()
    }

    pub fn print(self, limit: usize) {
        self.head.print(0, limit);
    }
}

