use inquire::{error::InquireError, Select, ui::{RenderConfig, Attributes, Color, Styled, StyleSheet}};
use std::{process::exit ,fs::{self}, path::{PathBuf}};
use itertools::Itertools;
use jwalk::{
    rayon::prelude::{IntoParallelRefIterator, ParallelBridge, ParallelIterator},
    WalkDir,
};

pub struct Cleaner {
    path: PathBuf,
    dirs: Vec<(PathBuf, usize)>,
    threads: usize,
}

fn recurse_file() -> std::io::Result<Vec<String>> {
    let mut buf = vec![];

    let entries = std::fs::read_dir("./")?;

    for entry in entries {
        let entry = entry?;
        buf.push(entry.path().to_string_lossy().to_string());
    }
    Ok(buf)
}

pub fn read_arg() -> String {
    let option = recurse_file(); //vector off all folders and file in the current directory

    let style_sheet = StyleSheet::default()
    .with_fg(Color::LightCyan)
    .with_attr(Attributes::ITALIC); //style sheet for slected options

    let styled_option = Styled::new("->").with_style_sheet(style_sheet); //make a new style prefix for currently highlighted option

    let answer: Result<String, InquireError> = Select::new("Select a folder or file to delete", option.expect("\n\x1b[31mError\x1b[0m something went wrong dang it\n"))
    .with_page_size(20)
    .with_help_message("Please choose wisely!")
    .with_render_config(RenderConfig::default().with_highlighted_option_prefix(styled_option).with_selected_option(Some(style_sheet)).with_scroll_up_prefix(Styled::new("^").with_style_sheet(style_sheet)).with_scroll_down_prefix(Styled::new("v").with_style_sheet(style_sheet)))
    .prompt();

    match answer {
        Ok(path) => path,
        Err(e) => {
            println!("\n\x1b[31mError\x1b[0m Something went wrong lol \x1b[1m\x1b[33m{e}\x1b[0m");
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
