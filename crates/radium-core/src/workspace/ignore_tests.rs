//! Unit tests for ignore pattern handling.

#[cfg(test)]
mod tests {
    use super::super::*;
    use tempfile::TempDir;
    use std::fs;

    #[test]
    fn test_ignore_walker_respects_gitignore() {
        let temp_dir = TempDir::new().unwrap();
        let root = temp_dir.path();

        // Initialize git repo for .gitignore to work
        std::process::Command::new("git")
            .arg("init")
            .arg("--quiet")
            .current_dir(root)
            .output()
            .ok();

        fs::write(root.join(".gitignore"), "*.log\ntemp/").unwrap();
        fs::write(root.join("file.rs"), "content").unwrap();
        fs::write(root.join("file.log"), "content").unwrap();
        fs::create_dir_all(root.join("temp")).unwrap();
        fs::write(root.join("temp/file.txt"), "content").unwrap();

        let walker = ignore::IgnoreWalker::new(root);
        let mut paths: Vec<_> = walker.build().collect();
        paths.sort();

        // Should only include file.rs, not file.log or temp/file.txt
        let rs_files: Vec<_> = paths.iter()
            .filter(|p| p.extension().and_then(|e| e.to_str()) == Some("rs"))
            .collect();
        assert_eq!(rs_files.len(), 1);
        assert!(rs_files[0].ends_with("file.rs"));
    }

    #[test]
    fn test_ignore_walker_excludes_common_dirs() {
        let temp_dir = TempDir::new().unwrap();
        let root = temp_dir.path();

        fs::create_dir_all(root.join("target")).unwrap();
        fs::create_dir_all(root.join("node_modules")).unwrap();
        fs::write(root.join("target/file.rs"), "content").unwrap();
        fs::write(root.join("node_modules/package.json"), "{}").unwrap();
        fs::create_dir_all(root.join("src")).unwrap();
        fs::write(root.join("src/file.rs"), "content").unwrap();

        let walker = ignore::IgnoreWalker::new(root);
        let mut paths: Vec<_> = walker.build().collect();
        paths.sort();

        // Should only include src/file.rs
        let rs_files: Vec<_> = paths.iter()
            .filter(|p| p.extension().and_then(|e| e.to_str()) == Some("rs"))
            .collect();
        assert_eq!(rs_files.len(), 1);
        assert!(rs_files[0].ends_with("src/file.rs"));
    }

    #[test]
    fn test_should_ignore_path() {
        assert!(ignore::should_ignore_path(std::path::Path::new("target/file.rs")));
        assert!(ignore::should_ignore_path(std::path::Path::new("src/node_modules/file.js")));
        assert!(!ignore::should_ignore_path(std::path::Path::new("src/file.rs")));
    }
}
