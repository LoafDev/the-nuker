use inquire::Text;
use std::{fs, path::PathBuf};

use itertools::Itertools;
use jwalk::{
    rayon::prelude::{IntoParallelRefIterator, ParallelBridge, ParallelIterator},
    WalkDir,
};

use std::process::exit;

mod copied_autocomple;

pub struct Cleaner {
    path: PathBuf,
    dirs: Vec<(PathBuf, usize)>,
    threads: usize,
}

pub fn read_arg() -> String {
    let dir = Text::new("\nThe_nuker is a program to execute your file (hopefully) quickly original code was made by Dillon Beliveau on github: https://github.com/Dillonb/nmuidi . To get started type a directory to delete\n\x1b[1m\x1b[33mEnter\x1b[0m folder's directory to delete\n")
        .with_help_message("Use arrow keys to navigate folders")
        .with_autocomplete(copied_autocomple::FilePathCompleter::default())
        .prompt();

    match dir {
        Ok(dir) => dir,
        Err(_) => {
            println!("\n\x1b[31mError\x1b[0m Something went wrong but I don't know what went wrong lol");
            exit(1);
        }
    }
}

impl Cleaner {
    pub fn new<T>(path: T) -> Self
    where
        PathBuf: std::convert::From<T>,
    {
        Self {
            path: path.into(),
            dirs: Vec::new(),
            threads: num_cpus::get() * 100,
        }
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
                        exit(1);
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
