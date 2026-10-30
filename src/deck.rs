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
    pub entries: Vec<Entry>,
    pub size: u64,
}

static POOL: LazyLock<ThreadPool> = LazyLock::new(|| ThreadPool::new(10));

impl Folder {
    fn new(dir: PathBuf) -> Self {
        Self {
            dir,
            entries: vec![],
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
                        let mut file = File::new(entry.path());
                        file.size = md.size();
                        self.size += file.size;
                        self.entries.push(Entry::File(file));
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
                            self.entries.push(Entry::Folder(folder));
                        }
                    }
                }

                if !child {
                    drop(tx);
                    for folder in rx.iter() {
                        self.size += folder.size;
                        self.entries.push(Entry::Folder(folder));
                    }
                }
            }
            Err(e) => {
                eprintln!("Error: Could not read dir: {} {}", e, self.dir.display())
            }
        }

        Ok(self.size)
    }

    pub fn sort(&mut self) {
        self.entries.sort_by_key(|f| std::cmp::Reverse(f.size()));
    }
}

pub struct File {
    pub dir: PathBuf,
    pub size: u64,
}

impl File {
    pub fn new(dir: PathBuf) -> Self {
        Self { dir, size: 0 }
    }
}

pub enum Entry {
    Folder(Folder),
    File(File),
}

impl Entry {
    pub fn size(&self) -> u64 {
        match self {
            Entry::File(file) => file.size,
            Entry::Folder(folder) => folder.size,
        }
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
        self.head
            .entries
            .sort_by_key(|f| std::cmp::Reverse(f.size()));
        Ok(self.head.size)
    }

    pub fn current_folder(&self) -> Option<&Folder> {
        let mut current_folder = &self.head;

        for &index in &self.pointer {
            current_folder = match current_folder.entries.get(index) {
                Some(Entry::Folder(folder)) => folder,
                _ => return None,
            }
        }

        Some(current_folder)
    }

    pub fn current_folder_mut(&mut self) -> Option<&mut Folder> {
        let mut current = &mut self.head;

        for &index in &self.pointer {
            current = match current.entries.get_mut(index) {
                Some(Entry::Folder(folder)) => folder,
                _ => return None,
            }
        }
        Some(current)
    }
}
