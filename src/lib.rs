use lexopt::prelude::*;
use std::{fs, path::PathBuf};

use itertools::Itertools;
use jwalk::{
    rayon::prelude::{IntoParallelRefIterator, ParallelBridge, ParallelIterator},
    WalkDir,
};
use log::error;

pub struct Args {
    pub dir: String,
}

pub struct Cleaner {
    path: PathBuf,
    dirs: Vec<(PathBuf, usize)>,
    threads: usize,
}

pub fn read_arg() -> Result<Args, lexopt::Error> {
    let mut dir = None;
    let mut parser = lexopt::Parser::from_env();

    while let Some(arg) = parser.next()? {
        match arg {
            Value(val) if dir.is_none() => {
                dir = Some(val.string()?);
            }

            _ => return Err(arg.unexpected())
        }
    }

    Ok(Args{
        dir: dir.ok_or("The_nuker is a program to execute your file (hopefully) quickly original code was made by Dillon Beliveau on github: https://github.com/Dillonb/nmuidi . To get started type a directory to delete")?,
    })
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
                        println!("Error removing directory {}: {e}", dir.display());
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
                                error!("Error making {} write-accessable: {e}", path.display());
                            });
                        }
                        if f_type.is_file() || f_type.is_symlink() {
                            fs::remove_file(&path).unwrap_or_else(|e| {
                                error!("Failed to remove file {}: {e}", path.display());
                            });
                        } else if f_type.is_dir() {
                            return Some((path, entry.depth));
                        }
                    }
                    Err(error) => error!("Error processing directory entry: {error}"),
                }
                None
            })
            .collect();
        dirs.sort_by(|a, b| b.1.cmp(&a.1));
        self.dirs = dirs;
    }
}
