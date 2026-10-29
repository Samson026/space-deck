use std::{
    fs::{self},
    io::Result,
    os::unix::fs::MetadataExt,
    path::PathBuf,
    sync::{LazyLock, mpsc::channel},
};

use threadpool::ThreadPool;

#[derive(Default)]
pub struct Folder {
    pub dir: PathBuf,
    pub folders: Vec<Folder>,
    pub files: Vec<PathBuf>,
    pub size: u64,
}

static POOL: LazyLock<ThreadPool> = LazyLock::new(|| ThreadPool::new(10));

impl Folder {
    fn new(dir: PathBuf) -> Self {
        Self {
            dir,
            folders: vec![],
            files: vec![],
            size: 0,
        }
    }

    fn size_cal(&mut self, child: bool) -> Result<u64> {
        let files = fs::read_dir(&self.dir);

        match files {
            Ok(files) => {
                let (tx, rx) = channel();
                for file in files {
                    let entry = file?;
                    let md = entry.metadata()?;

                    if md.is_file() {
                        self.size += md.size();
                        self.files.push(entry.path());
                    } else if md.is_dir() {
                        // let size_clone = Arc::clone(self.size);
                        if !child {
                            let tx = tx.clone();
                            POOL.execute(move || {
                                let mut folder = Folder::new(entry.path());
                                _ = folder.size_cal(true);
                                _ = tx.send(folder);
                            })
                        } else {
                            let mut folder = Folder::new(entry.path());
                            self.size += folder.size_cal(true)?;
                            self.folders.push(folder);
                        }
                    }
                }

                if !child {
                    drop(tx);
                    for folder in rx.iter() {
                        self.size += folder.size;
                        self.folders.push(folder);
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
    pub pointer: Vec<usize>,
}

impl SpaceDeck {
    pub fn new(dir: PathBuf) -> Self {
        let head = Folder::new(dir);
        Self {
            head,
            pointer: vec![],
        }
    }

    pub fn size_cal(&mut self) -> Result<u64> {
        self.head.size_cal(false)?;
        self.head.folders.sort_by_key(|f| std::cmp::Reverse(f.size));
        Ok(self.head.size)
    }

    pub fn print(self, limit: usize) {
        _ = self.head.print(0, limit);
    }

    pub fn current_folder(&self) -> Option<&Folder> {
        let mut current_folder = &self.head;

        for &index in &self.pointer {
            current_folder = current_folder.folders.get(index)?;
        }

        Some(current_folder)
    }
}
