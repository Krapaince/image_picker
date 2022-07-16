use super::item::CategoryTreeItem;
use crate::config::Category;
use csv::{ReaderBuilder, WriterBuilder};
use egui::CollapsingHeader;
use log::info;
use std::{
    collections::HashSet,
    error::Error,
    path::{Path, PathBuf},
};

pub struct CategoryTree {
    item: CategoryTreeItem,
    leafs: Vec<CategoryTree>,
}

impl CategoryTree {
    pub fn load_paths(&mut self, path: &Path) -> Result<(), Box<dyn Error>> {
        self.leafs
            .iter_mut()
            .try_for_each(|leaf| leaf.load_paths(path))?;

        let csv_path = self.item.make_category_path(path);
        let paths = if csv_path.exists() {
            let mut rdr = ReaderBuilder::new()
                .has_headers(false)
                .from_path(&csv_path)?;
            let paths: Vec<PathBuf> = rdr.deserialize().into_iter().collect::<Result<_, _>>()?;
            info!("Readed {} from {}", paths.len(), csv_path.display());
            paths
        } else {
            vec![]
        };
        self.item.set_paths(paths);

        Ok(())
    }

    pub fn get_paths(&self) -> HashSet<PathBuf> {
        let mut paths: HashSet<_> = self.item.get_paths().iter().cloned().collect();

        self.leafs
            .iter()
            .for_each(|leaf| paths.extend(leaf.get_paths()));
        paths
    }

    pub fn export_paths(&self, output_dir: &Path) -> Result<(), Box<dyn Error>> {
        self.export_paths_inner(output_dir, &mut HashSet::new())
    }

    fn export_paths_inner(
        &self,
        output_dir: &Path,
        parent_paths: &mut HashSet<PathBuf>,
    ) -> Result<(), Box<dyn Error>> {
        let mut item_paths = self
            .item
            .get_paths()
            .clone()
            .into_iter()
            .collect::<HashSet<_>>();

        self.leafs
            .iter()
            .try_for_each(|leaf| leaf.export_paths_inner(output_dir, &mut item_paths))?;

        let mut wdr = WriterBuilder::new()
            .has_headers(false)
            .from_path(self.item.make_category_path(output_dir))?;

        let item_paths = {
            let mut item_paths = item_paths.into_iter().collect::<Vec<_>>();
            item_paths.sort();
            item_paths
        };
        for path in &item_paths {
            wdr.serialize(&path)?;
        }

        parent_paths.extend(item_paths.into_iter());

        Ok(())
    }

    pub fn find_item_by_name(&mut self, category: &str) -> Option<&mut CategoryTreeItem> {
        if self.item.name() == category {
            Some(&mut self.item)
        } else {
            let mut find_leaf = None;

            for leaf in &mut self.leafs {
                find_leaf = leaf.find_item_by_name(category);

                if find_leaf.is_some() {
                    break;
                }
            }

            find_leaf
        }
    }

    pub fn update(
        &mut self,
        ctx: &egui::Context,
        ui: &mut egui::Ui,
        selected_cat: &mut Option<String>,
    ) {
        let text = format!("{} ({:?})", self.item.name(), self.item.key());

        if !self.leafs.is_empty() {
            CollapsingHeader::new(text)
                .default_open(true)
                .show(ui, |ui| {
                    self.leafs.iter_mut().for_each(|leaf| {
                        leaf.update(ctx, ui, selected_cat);
                    });
                });
        } else {
            ui.label(text);
        }

        if ctx.input().key_pressed(self.item.key()) {
            *selected_cat = Some(self.item.name().to_string());
        }
    }
}

impl From<Category> for CategoryTree {
    fn from(value: Category) -> Self {
        let leafs = if let Some(sub_categories) = value.sub_categories {
            let mut leafs = Vec::with_capacity(sub_categories.len());

            sub_categories
                .into_iter()
                .for_each(|category| leafs.push(category.into()));

            leafs
        } else {
            vec![]
        };

        Self {
            item: CategoryTreeItem::new(value.name, value.key),
            leafs,
        }
    }
}
