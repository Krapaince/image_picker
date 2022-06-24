use std::{
    collections::VecDeque,
    fs::File,
    io::{BufReader, Read},
    path::{Path, PathBuf},
    sync::mpsc::{self, Receiver},
    thread::{self, JoinHandle},
    vec::Vec,
};

use egui_extras::RetainedImage;
use log::{info, trace, warn};

pub struct Image {
    pub source: PathBuf,
    pub buffer: RetainedImage,
}

struct DataLoaderThread {
    handle: JoinHandle<()>,
    rx: Receiver<Option<Image>>,
}

pub struct DataLoader {
    paths: Vec<PathBuf>,
    threads: Vec<DataLoaderThread>,
    buffer: VecDeque<Image>,
}

impl DataLoader {
    pub fn new(capacity: usize, paths: Vec<PathBuf>) -> Self {
        let mut preloader = Self {
            paths,
            threads: Vec::with_capacity(capacity),
            buffer: VecDeque::with_capacity(capacity),
        };
        preloader.prefetch_images();
        preloader.await_fetched_images(Some(1));

        preloader
    }

    pub fn read_current(&mut self) -> Option<Image> {
        self.prefetch_images();
        self.collect_readed_images();
        if self.buffer.is_empty() {
            self.await_fetched_images(Some(1));
        }

        self.buffer.pop_back()
    }

    fn prefetch_image(&mut self) -> bool {
        if let Some(path) = self.paths.pop() {
            let (tx, rx) = mpsc::channel();

            let handle = thread::spawn(move || {
                tx.send(read_image(path)).unwrap();
            });

            self.threads.push(DataLoaderThread { handle, rx });

            true
        } else {
            false
        }
    }

    pub fn prefetch_images(&mut self) {
        let nb_image_to_prefetch = self
            .buffer
            .capacity()
            .saturating_sub(self.buffer.len() + self.threads.len());
        let mut nb_prefetch = 0;

        for _ in 0..nb_image_to_prefetch {
            if self.prefetch_image() {
                nb_prefetch += 1;
            } else {
                break;
            }
        }
        if nb_prefetch != 0 {
            info!("Prefething {} images.", nb_prefetch);
            info!("{} images in threads.", self.threads.len());
        }
    }

    fn collect_readed_images(&mut self) {
        let len = self.threads.len();
        drain_filter(
            &mut self.threads,
            |x| x.handle.is_finished(),
            |x| {
                if x.handle.join().is_err() {
                    warn!("Couldn't join");
                }

                match x.rx.recv() {
                    Ok(content) => {
                        if let Some(image) = content {
                            trace!("Adding {} to buffer", image.source.display());
                            self.buffer.push_front(image);
                        }
                    }
                    Err(err) => warn!("{}", err),
                };
            },
        );
        let pushed_images = len - self.threads.len();
        if pushed_images > 0 {
            info!("Pushed {} images from threads", pushed_images);
        }
    }

    fn await_fetched_images(&mut self, n: Option<usize>) {
        let mut n = match n {
            Some(n) => n,
            None => self.threads.len(),
        };

        drain_filter_n(
            &mut self.threads,
            &mut n,
            |n| n > 0,
            |x, n| {
                x.handle.join().unwrap();
                *n = n.saturating_sub(1);

                if let Ok(content) =  x.rx.recv() {
                    if let Some(image)= content  {
                        trace!("Adding {} to buffer", image.source.display());
                        self.buffer.push_front(image);
                    }
                }
            },
        );
    }
}

fn read_image(path: PathBuf) -> Option<Image> {
    let print_error = |path: &Path, err: String| {
        warn!(
            "Couldn't read {} because {}",
            path.display(),
            err.to_lowercase()
        );
    };
    trace!("Start reading {}", path.display());
    let mut buffer = Vec::new();
    let file = match File::open(&path) {
        Ok(file) => file,
        Err(err) => {
            print_error(&path, err.to_string());
            return None;
        }
    };

    if let Err(err) = BufReader::new(file).read_to_end(&mut buffer) {
        print_error(&path, err.to_string());
    }

    match RetainedImage::from_image_bytes(path.to_string_lossy(), &buffer) {
        Ok(buffer) => {
            trace!("Finnish reading {}", path.display());
            Some(Image {
                source: path,
                buffer,
            })
        }
        Err(err) => {
            print_error(&path, err);
            None
        }
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
