use std::{fs, path::Path};

use viewwitness::from_yaml;

#[test]
fn entire_example_corpus_parses_and_validates() {
    let mut files = Vec::new();
    collect_yaml(Path::new("examples"), &mut files);
    assert!(!files.is_empty(), "example corpus unexpectedly empty");

    for path in files {
        let source = fs::read_to_string(&path)
            .unwrap_or_else(|error| panic!("failed to read {}: {error}", path.display()));
        let witness = from_yaml(&source)
            .unwrap_or_else(|error| panic!("failed to parse {}: {error}", path.display()));
        let issues = witness.validation_issues();
        assert!(
            issues.is_empty(),
            "{} failed validation:\n{}",
            path.display(),
            issues.join("\n")
        );
    }
}

fn collect_yaml(dir: &Path, out: &mut Vec<std::path::PathBuf>) {
    for entry in fs::read_dir(dir)
        .unwrap_or_else(|error| panic!("failed to read {}: {error}", dir.display()))
    {
        let path = entry.expect("directory entry").path();
        if path.is_dir() {
            collect_yaml(&path, out);
        } else if path
            .extension()
            .is_some_and(|extension| extension == "yaml")
        {
            out.push(path);
        }
    }
}
