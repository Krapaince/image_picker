use std::{
    collections::VecDeque,
    fs::File,
    io::{BufReader, Read},
    path::PathBuf,
    sync::mpsc::{self, Receiver},
    thread::{self, JoinHandle},
    vec::Vec,
};

use egui_extras::RetainedImage;
use log::{info, trace, warn};

pub struct Image {
    pub source: PathBuf,
    pub buffer: Result<RetainedImage, ()>,
}

struct DataLoaderThread {
    handle: JoinHandle<()>,
    rx: Receiver<Image>,
}

pub struct DataLoader {
    paths: Vec<PathBuf>,
    index: usize,
    current_index: usize,
    capacity: usize,
    threads: Vec<DataLoaderThread>,
    buffer: VecDeque<Image>,
}

impl DataLoader {
    pub fn new(capacity: usize, paths: Vec<PathBuf>) -> Self {
        let mut preloader = Self {
            paths,
            index: 0,
            current_index: 0,
            capacity,
            threads: Vec::with_capacity(capacity),
            buffer: VecDeque::with_capacity(capacity),
        };
        preloader.read_images(capacity);
        preloader
    }

    fn read_images(&mut self, n: usize) {
        for _ in 0..n {
            if let Some(path) = self.paths.get(self.index).cloned() {
                self.index += 1;
                self.buffer.push_front(read_image(path));
            } else {
                break;
            }
        }
    }

    pub fn read_current(&mut self) -> Option<Image> {
        if self.buffer.is_empty() && self.current_index < self.paths.len() {
            if self.threads.is_empty() {
                self.prefetch_images();
            }
            self.wait_for_images(Some(5));
        }
        self.push_readed_images();

        let element = self.buffer.pop_back();
        if element.is_some() {
            self.current_index += 1;
        }

        if self.buffer.len() < self.capacity / 2 {
            self.prefetch_images()
        }

        element
    }

    pub fn prefetch_images(&mut self) {
        let nb_image_to_prefetch = self
            .buffer
            .capacity()
            .saturating_sub(self.buffer.len() + self.threads.len());

        info!("Prefething {} images.", nb_image_to_prefetch);
        info!("{} images in threads.", self.threads.len());
        for _ in 0..nb_image_to_prefetch {
            if let Some(path) = self.paths.get(self.index).cloned() {
                self.index += 1;
                let (tx, rx) = mpsc::channel();

                let handle = thread::spawn(move || {
                    tx.send(read_image(path)).unwrap();
                });

                self.threads.push(DataLoaderThread { handle, rx });
            } else {
                break;
            }
        }
    }

    fn push_readed_images(&mut self) {
        let len = self.threads.len();
        drain_filter(
            &mut self.threads,
            |x| x.handle.is_finished(),
            |x| {
                x.handle.join();

                if let Ok(image) = x.rx.recv() {
                    trace!("Adding {} to buffer", image.source.display());
                    self.buffer.push_front(image);
                }
            },
        );
        info!("Pushed {} images from threads", len - self.threads.len());
    }

    fn wait_for_images(&mut self, n: Option<usize>) {
        let mut n = if let Some(n) = n {
            n
        } else {
            self.threads.len()
        };

        drain_filter_n(
            &mut self.threads,
            &mut n,
            |n| n > 0,
            |x, n| {
                x.handle.join().unwrap();
                *n = n.saturating_sub(1);

                if let Ok(image) = x.rx.recv() {
                    trace!("Adding {} to buffer", image.source.display());
                    self.buffer.push_front(image);
                }
            },
        )
    }

    pub fn current(&self) -> usize {
        self.current_index
    }

    pub fn len(&self) -> usize {
        self.paths.len()
    }
}

fn read_image(path: PathBuf) -> Image {
    trace!("Start reading {}", path.display());
    let f = File::open(&path).unwrap();
    let mut reader = BufReader::new(f);
    let mut buffer = Vec::new();

    reader.read_to_end(&mut buffer).unwrap();

    let buffer = match RetainedImage::from_image_bytes(path.to_string_lossy(), &buffer) {
        Ok(buffer) => {
            trace!("Finnish reading {}", path.display());
            Ok(buffer)
        }
        Err(err) => {
            warn!("Couldn't read {} because {}", path.display(), err);
            Err(())
        }
    };

    Image {
        source: path,
        buffer,
    }
}

fn drain_filter<T, P, F>(vec: &mut Vec<T>, mut pred: P, mut f: F)
where
    P: FnMut(&T) -> bool,
    F: FnMut(T),
{
    let mut i = 0;

    while i < vec.len() {
        if pred(&vec[i]) {
            f(vec.remove(i));
        } else {
            i += 1;
        }
    }
}

fn drain_filter_n<T, P, F>(vec: &mut Vec<T>, n: &mut usize, mut pred: P, mut f: F)
where
    P: FnMut(usize) -> bool,
    F: FnMut(T, &mut usize),
{
    let mut i = 0;

    while i < vec.len() {
        if pred(*n) {
            f(vec.remove(i), n);
        } else {
            i += 1;
        }
    }
}
