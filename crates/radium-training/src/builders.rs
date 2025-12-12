use crate::dataset::{compute_dataset_id, validate_examples, Dataset, DatasetId, DatasetSource, ScanDepth, TrainingExample};
use crate::error::{TrainingError, TrainingResult};
use ignore::WalkBuilder;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone)]
pub struct DatasetBuildOptions {
    pub max_files: usize,
    pub max_examples: usize,
    pub max_bytes_per_file: u64,
    pub max_chars_per_example: usize,
    pub min_chars_per_file: usize,
    pub include_extensions: Vec<&'static str>,
}

impl Default for DatasetBuildOptions {
    fn default() -> Self {
        Self {
            max_files: 200,
            max_examples: 1000,
            max_bytes_per_file: 1_000_000, // 1MB
            max_chars_per_example: 4000,
            min_chars_per_file: 200,
            include_extensions: vec![
                "md", "txt",
                "rs", "toml",
                "ts", "tsx", "js", "jsx", "json",
                "py",
                "go",
                "java",
                "rb",
            ],
        }
    }
}

/// Build a dataset from a `DatasetSource`.
///
/// v1 policy: generate **self-supervised** SFT-ish examples from local text by splitting
/// file content into `(prompt, response)` pairs. This requires no LLM labeling step and
/// is suitable for local-only workflows.
pub fn build_dataset(source: &DatasetSource, options: &DatasetBuildOptions) -> TrainingResult<(Dataset, DatasetId)> {
    let examples = match source {
        DatasetSource::RepoScan { root, depth } => build_from_repo(root, *depth, options)?,
        DatasetSource::TextFiles { paths } => build_from_paths(paths, options)?,
        DatasetSource::Jsonl { path } => read_jsonl_dataset(path, options)?,
    };

    validate_examples(&examples)?;
    let id = compute_dataset_id(&examples)?;
    Ok((examples, id))
}

pub fn write_jsonl_dataset(path: &Path, examples: &[TrainingExample]) -> TrainingResult<()> {
    let mut out = String::new();
    for ex in examples {
        out.push_str(&serde_json::to_string(ex)?);
        out.push('\n');
    }
    std::fs::write(path, out)?;
    Ok(())
}

pub fn read_jsonl_dataset(path: &Path, options: &DatasetBuildOptions) -> TrainingResult<Dataset> {
    let contents = std::fs::read_to_string(path)?;
    let mut dataset = Vec::new();

    for (idx, line) in contents.lines().enumerate() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        let ex: TrainingExample = serde_json::from_str(line).map_err(|e| {
            TrainingError::Dataset(format!("failed to parse jsonl line {}: {}", idx + 1, e))
        })?;
        dataset.push(ex);
        if dataset.len() >= options.max_examples {
            break;
        }
    }

    Ok(dataset)
}

fn build_from_repo(root: &Path, depth: ScanDepth, options: &DatasetBuildOptions) -> TrainingResult<Dataset> {
    if !root.exists() {
        return Err(TrainingError::Dataset(format!("repo root does not exist: {}", root.display())));
    }

    let max_files = match depth {
        ScanDepth::Quick => options.max_files.min(60),
        ScanDepth::Full => options.max_files,
    };

    let paths = collect_text_files(root, max_files, options)?;
    build_examples_from_files(&paths, options)
}

fn build_from_paths(paths: &[PathBuf], options: &DatasetBuildOptions) -> TrainingResult<Dataset> {
    if paths.is_empty() {
        return Err(TrainingError::Dataset("text file paths must not be empty".to_string()));
    }

    let mut files = Vec::new();
    for p in paths {
        if p.is_dir() {
            let collected = collect_text_files(p, options.max_files, options)?;
            files.extend(collected);
        } else {
            files.push(p.clone());
        }
    }

    // De-dup and cap
    files.sort();
    files.dedup();
    files.truncate(options.max_files);

    build_examples_from_files(&files, options)
}

fn collect_text_files(root: &Path, max_files: usize, options: &DatasetBuildOptions) -> TrainingResult<Vec<PathBuf>> {
    let mut files = Vec::new();

    // Use ignore crate for .gitignore support and standard exclusions
    let mut builder = WalkBuilder::new(root);
    builder.follow_links(false);
    builder.add_custom_ignore_filename(".nxignore");

    let walker = builder.build();

    for entry in walker {
        let entry = match entry {
            Ok(e) => e,
            Err(_) => continue,
        };

        if !entry.file_type().map(|ft| ft.is_file()).unwrap_or(false) {
            continue;
        }

        let path = entry.path();
        if !is_allowed_extension(path, &options.include_extensions) {
            continue;
        }

        // Skip large files
        if let Ok(md) = path.metadata() {
            if md.len() > options.max_bytes_per_file {
                continue;
            }
        }

        files.push(path.to_path_buf());
        if files.len() >= max_files {
            break;
        }
    }

    Ok(files)
}

fn is_allowed_extension(path: &Path, allowed: &[&'static str]) -> bool {
    let Some(ext) = path.extension().and_then(|e| e.to_str()) else {
        return false;
    };
    let ext = ext.to_lowercase();
    allowed.iter().any(|a| *a == ext)
}

fn build_examples_from_files(files: &[PathBuf], options: &DatasetBuildOptions) -> TrainingResult<Dataset> {
    let mut examples = Vec::new();

    for path in files {
        if examples.len() >= options.max_examples {
            break;
        }

        let content = match std::fs::read_to_string(path) {
            Ok(c) => c,
            Err(_) => continue,
        };

        let content = content.replace("\r\n", "\n");
        if content.chars().count() < options.min_chars_per_file {
            continue;
        }

        let rel = path.to_string_lossy().to_string();
        let prefix = format!("File: {rel}\n\n");

        // Make one or more examples per file by chunking and splitting each chunk.
        let chars: Vec<char> = content.chars().collect();
        let total = chars.len();
        let chunk_len = options.max_chars_per_example * 2; // prompt half + response half

        // If the file is small enough, emit a single (prompt, response) split.
        if total <= chunk_len {
            let split = (total / 2).max(1);
            if split < total {
                let prompt: String = chars[..split].iter().collect();
                let response: String = chars[split..].iter().collect();
                if !prompt.trim().is_empty() && !response.trim().is_empty() {
                    examples.push(TrainingExample {
                        prompt: format!("{prefix}{prompt}"),
                        response,
                        metadata: serde_json::json!({
                            "source_path": rel,
                            "offset_chars": 0,
                        }),
                    });
                }
            }
            continue;
        }

        let mut start = 0;
        while start + options.max_chars_per_example < total && examples.len() < options.max_examples {
            let end = (start + chunk_len).min(total);
            let chunk: String = chars[start..end].iter().collect();

            let split = (chunk.chars().count() / 2).max(1);
            let prompt: String = chunk.chars().take(split).collect();
            let response: String = chunk.chars().skip(split).collect();

            if prompt.trim().is_empty() || response.trim().is_empty() {
                break;
            }

            examples.push(TrainingExample {
                prompt: format!("{prefix}{prompt}"),
                response,
                metadata: serde_json::json!({
                    "source_path": rel,
                    "offset_chars": start,
                }),
            });

            start += split;
        }
    }

    if examples.is_empty() {
        return Err(TrainingError::Dataset("no training examples generated (inputs too small or filtered out)".to_string()));
    }

    Ok(examples)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_build_text_dataset_from_dir() {
        let temp = TempDir::new().unwrap();
        let root = temp.path();
        std::fs::write(root.join("a.txt"), "hello world\n".repeat(200)).unwrap();
        std::fs::write(root.join("b.md"), "markdown\n".repeat(200)).unwrap();

        let options = DatasetBuildOptions { max_files: 10, max_examples: 50, ..Default::default() };
        let (ds, id) = build_dataset(
            &DatasetSource::TextFiles { paths: vec![root.to_path_buf()] },
            &options,
        )
        .unwrap();

        assert!(!ds.is_empty());
        assert!(!id.0.is_empty());
    }
}

