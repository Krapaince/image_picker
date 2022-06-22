use std::path::{Path, PathBuf};

use egui::Key;

pub struct CategoryTreeItem {
    name: String,
    key: Key,
    paths: Vec<PathBuf>,
}

impl CategoryTreeItem {
    pub fn new(name: String, key: Key) -> Self {
        Self {
            name,
            key,
            paths: vec![],
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn set_paths(&mut self, paths: Vec<PathBuf>) {
        self.paths = paths;
    }

    pub fn get_paths(&self) -> &Vec<PathBuf> {
        &self.paths
    }

    pub fn key(&self) -> Key {
        self.key
    }

    pub fn add_path(&mut self, path: PathBuf) {
        self.paths.push(path);
    }

    pub fn make_category_path(&self, path: &Path) -> PathBuf {
        let mut path = PathBuf::from(path);
        path.push(&self.name);
        path.set_extension("csv");

        path
    }
}
