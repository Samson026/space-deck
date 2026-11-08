use std::{
    fs::{self},
    io::Result,
    os::unix::fs::MetadataExt,
    path::PathBuf,
    sync::{Arc, LazyLock, RwLock, mpsc::channel},
    time::Duration,
};

use threadpool_executor::{
    ThreadPool,
    threadpool::{Builder, ExceedLimitPolicy},
};

use crate::progress::Progress;

const POOL_SIZE: usize = 10;
const MAX_POOL_SIZE: usize = 16;

#[derive(Default)]
pub struct Folder {
    pub dir: PathBuf,
    pub entries: Vec<Entry>,
    pub size: u64,
    pub progress: Arc<RwLock<Progress>>,
}

static POOL: LazyLock<ThreadPool> = LazyLock::new(|| {
    Builder::new()
        .core_pool_size(POOL_SIZE)
        .maximum_pool_size(MAX_POOL_SIZE)
        .keep_alive_time(Duration::from_secs(10))
        .exeed_limit_policy(ExceedLimitPolicy::Reject)
        .build()
});

impl Folder {
    fn new(dir: PathBuf, progress: Arc<RwLock<Progress>>) -> Self {
        Self {
            dir,
            entries: vec![],
            size: 0,
            progress,
        }
    }

    fn size_cal(&mut self) -> Result<u64> {
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
                        self.progress
                            .write()
                            .unwrap()
                            .errors
                            .push(format!("Info: read dir: {}", file.dir.display()));
                        self.progress.write().unwrap().progress += file.size;
                        self.entries.push(Entry::File(file));
                    } else if md.is_dir() {
                        // let size_clone = Arc::clone(self.size);

                        let tx = tx.clone();
                        let new_progress = Arc::clone(&self.progress);
                        let path = entry.path();
                        if POOL
                            .execute(move || {
                                let mut folder = Folder::new(path, new_progress);
                                _ = folder.size_cal();
                                _ = tx.send(folder);
                            })
                            .is_err()
                        {
                            let new_progress = Arc::clone(&self.progress);
                            let mut folder = Folder::new(entry.path(), new_progress);
                            self.size += folder.size_cal()?;
                            self.entries.push(Entry::Folder(folder))
                        }
                    }
                }

                drop(tx);
                for folder in rx.iter() {
                    self.size += folder.size;
                    self.entries.push(Entry::Folder(folder));
                }
            }
            Err(e) => self.progress.write().unwrap().errors.push(format!(
                "Error: could not read dir: {} {}",
                e,
                self.dir.display()
            )),
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

    pub fn dir(&self) -> &PathBuf {
        match self {
            Entry::File(file) => &file.dir,
            Entry::Folder(folder) => &folder.dir,
        }
    }
}

#[derive(Default)]
pub struct SpaceDeck {
    pub head: Folder,
    pub pointer: Vec<usize>,
    pub reading: bool,
    progress: Arc<RwLock<Progress>>,
}

impl SpaceDeck {
    pub fn new(dir: PathBuf, progress: Arc<RwLock<Progress>>) -> Self {
        let new_progress = Arc::clone(&progress);
        let head = Folder::new(dir, new_progress);

        progress.write().unwrap().total = head.dir.metadata().unwrap().size();

        Self {
            head,
            pointer: vec![],
            reading: false,
            progress,
        }
    }

    pub fn size_cal(&mut self) -> Result<u64> {
        self.reading = true;
        let result = self.head.size_cal();
        self.reading = false;

        result?;
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
