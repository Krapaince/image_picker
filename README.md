# Image picker

A GUI app to make sets of images (made for data science purpose).
![](https://previews.dropbox.com/p/thumb/ABnUWa9KZhJu1HOjyCadHuW7tSGt6eBZROjIscAEY49EmbIvvyvUq2vZa62Fl23szL6iyYJ16i2APMfVN35omfiTewatyuxBu5Qe7X6WmaN3H8nmLjvUsjglfAdXFgfca3gWMIHq-3vzzqtCOJROP2yeIHt2pIVvRvI9NzDnl86ynbzZBACS9qrz_11QAUnPLR8oVK0oHElrKyBcR-0vTObj7ScGhrkYfcmgjR90xTYXoLyJ_jwcoQCqMMGOIwDBHvXizmSvU-9dc8mwRcT0cQ2rtUdJcr3RI8wBnUkvrV35nE6CeUYgJ_jnl2Pr04i2pONnjijwWsLrsPVD4pufHZG7hKLp29n5oSfBB-L-w9qheDPATytktNAGk3EcODKEXtU/p.png)

It reads a set of images from a directory or a CSV file, and waits for the
user's input to categorized an image. Each category is link to a key (see
Config). The set of categories is defined by the user. Once every images have
been categorized, the app exports each category to a CSV in the output
directory specified in the config file.

The app comes with an autosave on quit so that big datasets can be made in
multiple sessions. The autosave feature prematurely exports categorized images
to their CSVs (in the output directory). When relaunch, the app loads the list
of images to categorized and removes those already categorized. For this
feature to work, the config file needs to stay the same.

## Build

### Requirements
  - `rust`/`cargo`
    - See [Install Rust](https://www.rust-lang.org/tools/install)


## Config

In order to run the app, a `config.json` must be in your current working
directory. The config is composed of three sub entries:
  - `input` (object):

    One of:
    - `ds` (string): A path to a CSV file.
    - `root` (string): A path to a directory.

      When `root` is specified the app reads images in the child directories of
      `root`.

  - `output_dir` (string): A path to a directory where the categories' CSV will be exported.
  - `categories` (array of category object):

    A category is composed of two mandatory elements + one optional:
    - `name` (string): Category's name (**Must be unique**)
    - `key` (string): Category's key binding (**Must be unique**)
    - Optional `sub_categories` (array of category object):

      When an image is added to a subcategory it's also added to its parent.

  Config example:
  ```json
  {
    "input": {
      "root": "/path/to/your/dataset"
    },
    "output_dir": "/output/directory/path",
    "categories": [
      {
        "name": "wanted",
        "key": "Y",
        "sub_categories": [
          {
            "name": "very wanted",
            "key": "U",
            "sub_categories": [
              {
                "name": "very very wanted",
                "key": "P"
              }
            ]
          }
        ]
      },
      {
        "name": "doubtful",
        "key": "D"
      },
      {
        "name": "unwanted",
        "key": "N"
      }
    ]
  }
  ```
