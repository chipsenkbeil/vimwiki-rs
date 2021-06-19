use std::{ffi::OsStr, fs, path::PathBuf};
use vimwiki::*;
use walkdir::WalkDir;

fn base_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/output")
}

macro_rules! find_file_pairs {
    ($in:expr, $out:expr) => {
        WalkDir::new(base_path())
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| e.path().extension() == Some(OsStr::new($in)))
            .filter(|e| {
                !e.path()
                    .file_stem()
                    .and_then(OsStr::to_str)
                    .unwrap()
                    .ends_with(".out")
            })
            .map(|e| (e.path().to_path_buf(), e.path().with_extension($out)))
            .filter(|(_, p)| p.exists())
    };
}

macro_rules! test_file_pairs {
    ($in:expr, $out:expr, $convert:expr) => {
        for (path_in, path_out) in find_file_pairs!($in, $out) {
            println!("Loading {}...", path_in.to_string_lossy());
            let in_str = fs::read_to_string(path_in).unwrap();

            println!("Loading {}...", path_out.to_string_lossy());
            let out_str = fs::read_to_string(path_out).unwrap();

            println!("Parsing input...");
            let language = Language::from_vimwiki_str(&in_str);
            let page: Page = language.parse().unwrap();

            println!("Converting...");
            let actual = $convert(page);

            similar_asserts::assert_str_eq!(actual, out_str);
        }
    };
}

#[test]
fn check_vimwiki_to_vimwiki() {
    test_file_pairs!("wiki", "out.wiki", |page: Page| page
        .to_vimwiki_string(Default::default())
        .unwrap());
}

#[cfg(feature = "html")]
#[test]
fn check_vimwiki_to_html() {
    test_file_pairs!("wiki", "out.html", |page: Page| page
        .to_html_string(Default::default())
        .unwrap());
}
