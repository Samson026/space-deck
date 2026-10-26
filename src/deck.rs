use std::{fs::{self, DirEntry, File, metadata}, io::Result, os::unix::fs::MetadataExt, path::PathBuf};

struct Folder {
    dir: PathBuf,
    folders: Vec<Folder>,
    files: Vec<DirEntry>,
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
                    self.files.push(entry);
                }
                else if md.is_dir() {
                    let mut folder = Folder::new(entry.path());
                    self.size += folder.size_cal()?;
                    self.folders.push(folder);
                }
        }
            },
            Err(e) => {
                eprintln!("Error: Could not read dir")
            }
        }

        Ok(self.size)
    }

    fn print(self, depth: usize) -> Result<()>{
        if let Some(file) = self.dir.file_name() {
            let d = "-".repeat(depth) + &file.to_string_lossy() + ": " + &self.size.to_string();
            println!("{d}");
        }

        

        // for entry in self.files {
        //     let name = entry.file_name().to_string_lossy().into_owned();
        //     let output = "-".repeat(depth+1) + " " + &name;
        //     println!("{output}");
        // }
        

        for folder in self.folders {
            folder.print(depth+2);
        }
        Ok(())
    }
}

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

    pub fn print(self) {
        self.head.print(0);
    }
}

