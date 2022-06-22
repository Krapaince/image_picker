use std::{collections::HashSet, error::Error, fs::read_dir, path::PathBuf};

use csv::Reader;
use egui::{ProgressBar, RichText, Visuals};
use log::info;

use crate::{
    category::CategoriesHolder,
    config::{Config, Input},
    data_prefetcher::{DataLoader, Image},
};

pub struct ImagePicker {
    current_image: Option<Image>,

    dataloader: DataLoader,

    category: CategoriesHolder,
    output_dir: PathBuf,
}

impl ImagePicker {
    pub fn from_config(
        cc: &eframe::CreationContext<'_>,
        config: Config,
    ) -> Result<Self, Box<dyn Error>> {
        cc.egui_ctx.set_visuals(Visuals::dark());

        let Config {
            input,
            output_dir,
            categories,
        } = config;

        std::fs::create_dir_all(&output_dir)?;

        let mut category_tree = CategoriesHolder::from(categories);
        category_tree.load_paths(&output_dir)?;

        let paths = make_image_list(input, category_tree.get_paths())?;

        Ok(Self {
            current_image: None,

            dataloader: DataLoader::new(25, paths),

            category: category_tree,
            output_dir,
        })
    }

    fn handle_current(&mut self) {
        if self.category.is_there_a_selected_category() {
            if let Some(image) = self.current_image.take() {
                self.category.add_path_to_selected_category(image.source);
                self.read_next_image();
                std::thread::sleep(std::time::Duration::from_millis(100));
            }
        }
    }

    fn read_next_image(&mut self) {
        self.current_image = self.dataloader.read_current();
    }

    fn compute_progression(&self) -> f32 {
        self.dataloader.current() as f32 / self.dataloader.len() as f32
    }

    fn compute_nb_images_remaning(&self) -> usize {
        self.dataloader.len() - self.dataloader.current()
    }
}

impl eframe::App for ImagePicker {
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        egui::SidePanel::new(egui::panel::Side::Left, "Categories tree")
            .resizable(true)
            .show(ctx, |ui| {
                ui.horizontal_wrapped(|ui| {
                    self.category.update(ctx, ui);
                });
            });
        egui::CentralPanel::default().show(ctx, |ui| {
            if let Some(image) = self.current_image.as_ref() {
                if let Ok(buffer) = &image.buffer {
                    ui.vertical_centered(|ui| {
                        ui.heading(image.source.clone().to_str().unwrap_or_default());
                        ui.label(RichText::new(format!(
                            "{} left",
                            self.compute_nb_images_remaning()
                        )));

                        ui.add(ProgressBar::new(self.compute_progression()));
                    });

                    ui.with_layout(
                        egui::Layout::top_down_justified(egui::Align::Center),
                        |ui| {
                            let factors = ui.available_size() / buffer.size_vec2();

                            buffer.show_scaled(ui, factors.min_elem());
                        },
                    );
                }
                self.handle_current();
            } else {
                self.read_next_image();

                if self.current_image.is_none() {
                    frame.quit();
                }
            }
        });
    }

    fn on_exit(&mut self, _gl: &eframe::glow::Context) {
        self.category
            .export_paths(&self.output_dir)
            .map_err(|e| println!("{:?}", e));
    }
}

fn make_image_list(
    input: Input,
    paths_to_exclude: HashSet<PathBuf>,
) -> Result<Vec<PathBuf>, Box<dyn Error>> {
    let mut paths = vec![];
    let mut nb_removed_paths = 0;

    match input {
        Input::Dir { root } => {
            info!("Loading images from directory: {}", root.display());

            for entry in read_dir(root)? {
                let entry = entry?;
                let path = entry.path();

                if path.is_dir() {
                    for entry in read_dir(path)? {
                        let entry = entry?;
                        let path = entry.path();

                        if paths_to_exclude.get(&path).is_some() {
                            nb_removed_paths += 1;
                        } else if path.is_file() {
                            paths.push(path)
                        }
                    }
                }
            }
        }
        Input::Csv { ds, root } => {
            let csv_path = if let Some(mut root) = root {
                root.push(ds);
                root
            } else {
                ds
            };

            info!("Loading images from CSV file: {}", csv_path.display());

            let mut rdr = Reader::from_path(csv_path)?;

            paths.extend(
                rdr.deserialize()
                    .into_iter()
                    .collect::<Result<Vec<PathBuf>, _>>()?
                    .into_iter()
                    .filter_map(|x| {
                        if !x.exists() {
                            info!("{}: doesn't exist. Skipping", x.display());

                            None
                        } else {
                            Some(x)
                        }
                    })
                    .collect::<Vec<PathBuf>>(),
            );
        }
    };

    info!(
        "Found {} images removed {} images already categorized.",
        paths.len() + nb_removed_paths,
        nb_removed_paths
    );

    Ok(paths)
}
