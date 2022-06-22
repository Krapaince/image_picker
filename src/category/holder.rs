use crate::config::Category;
use std::{
    collections::HashSet,
    error::Error,
    path::{Path, PathBuf},
};

use super::{item::CategoryTreeItem, tree::CategoryTree};

pub struct CategoriesHolder {
    categories: Vec<CategoryTree>,
    selected_category: Option<String>,
}

impl CategoriesHolder {
    fn find_item_by_name(&mut self, category_name: &str) -> Option<&mut CategoryTreeItem> {
        let mut item = None;

        for category in &mut self.categories {
            item = category.find_item_by_name(category_name);

            if item.is_some() {
                break;
            }
        }

        item
    }

    pub fn update(&mut self, ctx: &egui::Context, ui: &mut egui::Ui) {
        self.categories
            .iter_mut()
            .map(|category| category.update(ctx, ui, &mut self.selected_category))
            .collect()
    }

    pub fn get_paths(&self) -> HashSet<PathBuf> {
        self.categories
            .iter()
            .flat_map(|category| category.get_paths())
            .collect()
    }

    pub fn load_paths(&mut self, path: &Path) -> Result<(), Box<dyn Error>> {
        self.categories
            .iter_mut()
            .try_for_each(|category| category.load_paths(path))?;

        Ok(())
    }

    pub fn export_paths(&self, output_dir: &Path) -> Result<(), Box<dyn Error>> {
        self.categories
            .iter()
            .try_for_each(|category| category.export_paths(output_dir))
    }

    pub fn is_there_a_selected_category(&self) -> bool {
        self.selected_category.is_some()
    }

    pub fn add_path_to_selected_category(&mut self, path: PathBuf) -> bool {
        if let Some(category) = self.selected_category.take() {
            self.find_item_by_name(&category).unwrap().add_path(path);

            true
        } else {
            false
        }
    }
}

impl From<Vec<Category>> for CategoriesHolder {
    fn from(value: Vec<Category>) -> Self {
        Self {
            categories: value.into_iter().map(|x| x.into()).collect(),
            selected_category: None,
        }
    }
}
