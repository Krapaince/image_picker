use eframe::egui;
use env_logger::Builder;

mod app;
mod category;
mod config;
mod data_prefetcher;

use std::{convert::TryFrom, error::Error, path::Path};

use app::ImagePicker;
use config::Config;

fn main() -> Result<(), Box<dyn Error>> {
    let config = Config::try_from(Path::new("./config.json"))?;

    Builder::new().filter_level(log::LevelFilter::Info).init();

    let options = eframe::NativeOptions {
        initial_window_size: Some(egui::vec2(700.0, 700.0)),
        ..Default::default()
    };

    eframe::run_native(
        "Image picker for DeepLearning",
        options,
        Box::new(|cc| Box::new(ImagePicker::from_config(cc, config).unwrap())),
    );
}
