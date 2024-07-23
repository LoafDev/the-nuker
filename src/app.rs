use std::{fs, path::{Path, PathBuf}, io::Result, process::exit};
use itertools::Itertools;
use jwalk::{
    rayon::prelude::{IntoParallelRefIterator, ParallelBridge, ParallelIterator}, WalkDir
};
use ratatui::widgets::ListState;

pub enum CurrentScreen {
    ChoosePath,
    ConfirmPath,
    DeletedPath,
}

pub struct App {
    pub current_screen: CurrentScreen,
    pub list_state: ListState,
    pub file_list: Vec<PathBuf>,
    pub current_working_directory: PathBuf,
    pub path: PathBuf,
    pub dirs: Vec<(PathBuf, usize)>,
    pub threads: usize,
}

pub fn iter_file<P: AsRef<Path>>(dir: P) -> Result<Vec<PathBuf>> {
    fn inner(path: &Path) -> Result<Vec<PathBuf>> {
        let mut buf = Vec::<PathBuf>::new();
        let entries = std::fs::read_dir(path)?;
    
        for entry in entries {
            let entry = entry?;
            buf.push(entry.path());
        }
    
        Ok(buf)
    }
    inner(dir.as_ref())
}

impl App {
    pub fn new() -> Result<App> {
        let cwd = std::env::current_dir()?;
        let app = App {
            current_screen: CurrentScreen::ChoosePath,
            list_state: ListState::default(),
            file_list: iter_file(&cwd).expect("\x1b[1m\x1b[33mError:\x1b[0m Failed to parse file!"),
            current_working_directory: cwd,
            path: PathBuf::new(),
            dirs: Vec::new(),
            threads: num_cpus::get() * 100,
        };
        
        Ok(app)
    }

    pub fn clean(&mut self) {
        self.remove_files();
        self.remove_dirs();
    }

    fn remove_dirs(&mut self) {
        let dirs_by_depth = self.dirs.iter().group_by(|x| x.1);
        for (_, level) in &dirs_by_depth {
            level
                .collect::<Vec<_>>()
                .par_iter()
                .map(|(dir, _group)| dir)
                .for_each(|dir| {
                    if let Err(e) = fs::remove_dir_all(dir.as_path()) {
                        println!("\n\x1b[31mError\x1b[0m removing directory {}: {e}\n", dir.display());
                    }
                });
        }
    }

    fn remove_files(&mut self) {
        let mut dirs: Vec<(std::path::PathBuf, usize)> = WalkDir::new(&self.path)
            .skip_hidden(false)
            .parallelism(jwalk::Parallelism::RayonNewPool(self.threads))
            .into_iter()
            .par_bridge()
            .flat_map(|entry| {
                match entry {
                    Ok(entry) => {
                        let f_type = entry.file_type;
                        let path = entry.path();
                        let metadata = entry.metadata().unwrap();

                        let mut perm = metadata.permissions();
                        if perm.readonly() {
                            #[allow(clippy::permissions_set_readonly_false)]
                            perm.set_readonly(false);
                            fs::set_permissions(&path, perm).unwrap_or_else(|e| {
                                println!("\n\x1b[31mError\x1b[0m making {} write-accessable: {e}\n", path.display());
                                exit(1);
                            });
                        }
                        if f_type.is_file() || f_type.is_symlink() {
                            fs::remove_file(&path).unwrap_or_else(|e| {
                                println!("\n\x1b[31mFailed\x1b[0m to remove file {}: {e}\n", path.display());
                                exit(1);
                            });
                        } else if f_type.is_dir() {
                            return Some((path, entry.depth));
                        }
                    }
                    Err(error) => {
                        println!("\n\x1b[31mError\x1b[0m processing directory entry: {error}\n");
                        exit(1);
                    }
                }
                None
            })
            .collect();
        dirs.sort_by(|a, b| b.1.cmp(&a.1));
        self.dirs = dirs;
    }
}