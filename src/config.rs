use egui::Key;
use serde::Deserialize;
use std::{
    collections::HashMap,
    convert::TryFrom,
    error::Error,
    fmt::{self, Debug, Display},
    path::{Path, PathBuf},
};

#[derive(Deserialize)]
pub struct Category {
    pub name: String,
    pub key: Key,
    pub sub_categories: Option<Vec<Category>>,
}

impl Category {
    fn flattent_category(&self, result: &mut Vec<Category>) {
        if let Some(sub_categories) = &self.sub_categories {
            sub_categories
                .iter()
                .for_each(|x| x.flattent_category(result));
        }

        result.push(Category {
            name: self.name.clone(),
            key: self.key,
            sub_categories: None,
        });
    }
}

#[derive(Deserialize)]
#[serde(untagged)]
pub enum Input {
    Dir { root: PathBuf },
    Csv { ds: PathBuf, root: Option<PathBuf> },
}

#[derive(Deserialize)]
pub struct Config {
    pub input: Input,
    pub output_dir: PathBuf,
    pub categories: Vec<Category>,
}

impl Config {
    fn flatten_categories(&self) -> Vec<Category> {
        let mut categories = vec![];

        self.categories
            .iter()
            .for_each(|x| x.flattent_category(&mut categories));

        categories
    }

    fn check_uniqueness<T, U, F, G>(
        categories: &[Category],
        f: F,
        g: G,
    ) -> Result<(), Box<dyn Error>>
    where
        F: Fn(&Category, &mut HashMap<T, U>),
        G: Fn(HashMap<T, U>) -> Result<(), Box<dyn Error>>,
    {
        let bindings = {
            let mut bindings: HashMap<T, U> = HashMap::new();

            categories.iter().for_each(|x| {
                f(x, &mut bindings);
            });

            bindings
        };

        g(bindings)
    }

    fn check_name_uniqueness(categories: &[Category]) -> Result<(), Box<dyn Error>> {
        Self::check_uniqueness(
            categories,
            |x, bindings| {
                if let Some(binding) = bindings.get_mut(&x.name) {
                    *binding += 1;
                } else {
                    bindings.insert(x.name.clone(), 1);
                }
            },
            |bindings| {
                if let Some(binding) = bindings.into_iter().filter(|x| x.1 > 1).take(1).next() {
                    Err(Box::new(ConfigError::DuplicateName(binding)))
                } else {
                    Ok(())
                }
            },
        )
    }

    fn check_key_uniqueness(categories: &[Category]) -> Result<(), Box<dyn Error>> {
        Self::check_uniqueness::<Key, Vec<String>, _, _>(
            categories,
            |x, bindings| {
                if let Some(binding) = bindings.get_mut(&x.key) {
                    binding.push(x.name.clone());
                } else {
                    bindings.insert(x.key, vec![x.name.clone()]);
                }
            },
            |bindings| {
                if let Some(binding) = bindings
                    .into_iter()
                    .filter(|x| x.1.len() > 1)
                    .take(1)
                    .next()
                {
                    Err(Box::new(ConfigError::DuplicateBindings(binding)))
                } else {
                    Ok(())
                }
            },
        )
    }
}

impl TryFrom<&Path> for Config {
    type Error = Box<dyn Error>;

    fn try_from(value: &Path) -> Result<Self, Self::Error> {
        let data = std::fs::read_to_string(value)?;
        let config: Config = serde_json::from_str(&data)?;

        let categories = config.flatten_categories();
        Self::check_name_uniqueness(&categories)?;
        Self::check_key_uniqueness(&categories)?;
        Ok(config)
    }
}

pub enum ConfigError {
    DuplicateBindings((Key, Vec<String>)),
    DuplicateName((String, usize)),
}

impl Error for ConfigError {}

impl Debug for ConfigError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "{}", self)
    }
}

impl Display for ConfigError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::DuplicateBindings(e) => {
                writeln!(f, "Duplicate bindings {:?}", e.0)?;
                writeln!(f, "Due to categories:")?;
                write!(
                    f,
                    "{}",
                    e.1.iter().fold(String::new(), |acc, x| acc
                        + format!("  - {}\n", &x).as_str())
                )
            }
            Self::DuplicateName(e) => {
                write!(f, "Duplicate category name, got {} times \"{}\"", e.1, e.0)
            }
        }
    }
}
