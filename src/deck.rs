use std::{
    fs::{self}, io::Result, os::unix::fs::MetadataExt, path::PathBuf, sync::{Arc, Mutex}, thread,
};

#[derive(Default)]
pub struct Folder {
    pub dir: PathBuf,
    pub folders: Vec<Folder>,
    pub files: Vec<PathBuf>,
    pub size: u64,
}

impl Folder {
    fn new(dir: PathBuf) -> Self {
        Self {
            dir,
            folders: vec![],
            files: vec![],
            size: 0,
        }
    }

    fn size_cal(&mut self) -> Result<u64> {
        let files = fs::read_dir(&self.dir);

        match files {
            Ok(files) => {
                let handlers = vec![];
                for file in files {
                    let entry = file?;
                    let md = entry.metadata()?;

                    if md.is_file() {
    
                        *s += md.size();
                        self.files.push(entry.path());
                    } else if md.is_dir() {
                        let size_clone = Arc::clone(&self.size);
                        handlers.push(thread::spawn(move|| -> Result<Folder> {
                            let mut folder = Folder::new(entry.path());
                            folder.size_cal()?;
                            Ok(folder)
                        }))
                    }
                }
                
                
            }
            Err(e) => {
                eprintln!("Error: Could not read dir: {} {}", e, self.dir.display())
            }
        }

        Ok(self.size)
    }

    fn print(mut self, depth: usize, limit: usize) -> Result<()> {
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

        self.folders.sort_by_key(|f| std::cmp::Reverse(f.size));

        for folder in self.folders {
            _ = folder.print(depth + 1, limit);
        }
        Ok(())
    }
}

#[derive(Default)]
pub struct SpaceDeck {
    pub head: Folder,
}

impl SpaceDeck {
    pub fn new(dir: PathBuf) -> Self {
        let head = Folder::new(dir);
        Self { head }
    }

    pub fn size_cal(&mut self) -> Result<u64> {
        self.head.size_cal()
    }

    pub fn print(self, limit: usize) {
        _ = self.head.print(0, limit);
    }
}
