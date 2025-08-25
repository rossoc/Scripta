# Scripta

Scripta converts a directory of Markdown files to HTML files.
So your `readme.md` is converted into `index.html`. 
It recurses in every sub-directory.

```markdown
target
├── readme.md
├── folder1
│   ├── file1.md
│   ├── file2.md
│   └── folder2.md
│       ├── file3.md
│       ├── file4.md
│       └── ...
├── layout
│   ├── some_layout.md
│   └── ...
├── assets
│   ├── some_asset.jpg
│   └── ...
└── ...
```

Follows the steps through which the website is compiled:

1- Every directory, or file from the source directory to the output directory. 
  You can check out [file_walker](src/file_walker.rs), the function 
  `should_include` is used to determine which folder and file are needed.

2- Each folder is treated as a collection and all the Markdown in it are
   converted into HTML.

You can access the documentation with `cargo doc --open`.

The GitHub action on this page automatically run the cargo binary and pushes the
website online.

## Run the program locally

The site builder is written in Rust, hence, to run the code you can use:

- `cargo run`: it generate the folder `_site` inside the current and create the
  website there.
- `cargo run -- build -s <source_path>`: specify the folder to copy for the building 
  site.
- `cargo run -- build -o <output_path>`: specify the folder to create the site in.
- `cargo run -- build --watch`: check for updates on `<source_path>`.
- `cargo run -- build --serve`: serve output directory locally after building.
- `cargo run -- build --port`: port to run the server on.
- `cargo run -- write`: open today's note at the folder, if not existing it is
  created.
