use clap::{Parser, Subcommand};
use globset::{Glob, GlobSetBuilder};
use ignore::WalkBuilder;
use std::fmt::Write as FmtWrite;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Parser)]
#[command(
    name = "gather",
    about = "Fast context gathering for AI coding agents",
    long_about = "Gather files from a codebase and format them as structured context \
                  for AI coding agents. Respects .gitignore, supports glob filtering, \
                  and estimates token counts."
)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Collect file contents and output as structured context
    Collect {
        /// Root directory to gather from (defaults to current directory)
        #[arg(default_value = ".")]
        path: PathBuf,

        /// Glob patterns to include (can be specified multiple times)
        #[arg(short = 'g', long = "glob")]
        globs: Vec<String>,

        /// Glob patterns to exclude (can be specified multiple times)
        #[arg(short = 'e', long = "exclude")]
        excludes: Vec<String>,

        /// Maximum file size in bytes to include (default: 100KB)
        #[arg(long, default_value = "102400")]
        max_size: u64,

        /// Output format: markdown (default) or xml
        #[arg(short = 'f', long = "format", default_value = "markdown")]
        format: OutputFormat,

        /// Show token count estimate in output
        #[arg(long)]
        tokens: bool,
    },

    /// Show a tree view of the directory structure
    Tree {
        /// Root directory (defaults to current directory)
        #[arg(default_value = ".")]
        path: PathBuf,

        /// Glob patterns to include (can be specified multiple times)
        #[arg(short = 'g', long = "glob")]
        globs: Vec<String>,

        /// Glob patterns to exclude (can be specified multiple times)
        #[arg(short = 'e', long = "exclude")]
        excludes: Vec<String>,
    },

    /// Estimate token count for files without printing contents
    Tokens {
        /// Root directory (defaults to current directory)
        #[arg(default_value = ".")]
        path: PathBuf,

        /// Glob patterns to include (can be specified multiple times)
        #[arg(short = 'g', long = "glob")]
        globs: Vec<String>,

        /// Glob patterns to exclude (can be specified multiple times)
        #[arg(short = 'e', long = "exclude")]
        excludes: Vec<String>,

        /// Maximum file size in bytes to include (default: 100KB)
        #[arg(long, default_value = "102400")]
        max_size: u64,
    },
}

#[derive(Clone, Debug)]
enum OutputFormat {
    Markdown,
    Xml,
}

impl std::str::FromStr for OutputFormat {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "markdown" | "md" => Ok(OutputFormat::Markdown),
            "xml" => Ok(OutputFormat::Xml),
            _ => Err(format!("Unknown format: {s}. Use 'markdown' or 'xml'.")),
        }
    }
}

/// Estimate token count using a simple heuristic: ~4 characters per token.
/// This approximates GPT/Claude tokenization without needing a tokenizer library.
fn estimate_tokens(text: &str) -> usize {
    // Rough heuristic: 1 token ≈ 4 characters for English/code
    (text.len() + 3) / 4
}

/// Check if a file is likely binary by reading a small sample.
fn is_binary(path: &Path) -> bool {
    match fs::read(path) {
        Ok(bytes) => {
            let sample = &bytes[..bytes.len().min(8192)];
            sample.contains(&0)
        }
        Err(_) => true,
    }
}

/// Infer a markdown language tag from a file extension.
fn lang_tag(path: &Path) -> &str {
    match path.extension().and_then(|e| e.to_str()) {
        Some("rs") => "rust",
        Some("py") => "python",
        Some("js") => "javascript",
        Some("ts") => "typescript",
        Some("tsx") => "tsx",
        Some("jsx") => "jsx",
        Some("go") => "go",
        Some("rb") => "ruby",
        Some("java") => "java",
        Some("c") => "c",
        Some("cpp" | "cc" | "cxx") => "cpp",
        Some("h" | "hpp") => "cpp",
        Some("sh" | "bash") => "bash",
        Some("zsh") => "zsh",
        Some("fish") => "fish",
        Some("json") => "json",
        Some("yaml" | "yml") => "yaml",
        Some("toml") => "toml",
        Some("xml") => "xml",
        Some("html" | "htm") => "html",
        Some("css") => "css",
        Some("scss") => "scss",
        Some("sql") => "sql",
        Some("md") => "markdown",
        Some("dockerfile") => "dockerfile",
        Some("tf") => "hcl",
        Some("swift") => "swift",
        Some("kt" | "kts") => "kotlin",
        Some("r") => "r",
        Some("lua") => "lua",
        Some("zig") => "zig",
        Some("nix") => "nix",
        _ => "",
    }
}

struct CollectedFile {
    relative_path: String,
    content: String,
}

fn collect_files(
    root: &Path,
    globs: &[String],
    excludes: &[String],
    max_size: u64,
) -> Vec<CollectedFile> {
    let include_set = if globs.is_empty() {
        None
    } else {
        let mut builder = GlobSetBuilder::new();
        for g in globs {
            if let Ok(glob) = Glob::new(g) {
                builder.add(glob);
            }
        }
        builder.build().ok()
    };

    let exclude_set = if excludes.is_empty() {
        None
    } else {
        let mut builder = GlobSetBuilder::new();
        for g in excludes {
            if let Ok(glob) = Glob::new(g) {
                builder.add(glob);
            }
        }
        builder.build().ok()
    };

    let walker = WalkBuilder::new(root)
        .hidden(true) // skip hidden files
        .git_ignore(true) // respect .gitignore
        .git_global(true)
        .git_exclude(true)
        .build();

    let mut files = Vec::new();

    for entry in walker.flatten() {
        let path = entry.path();

        if !path.is_file() {
            continue;
        }

        // Check file size
        if let Ok(meta) = path.metadata() {
            if meta.len() > max_size {
                continue;
            }
        }

        let rel = path
            .strip_prefix(root)
            .unwrap_or(path)
            .to_string_lossy()
            .to_string();

        // Apply include globs
        if let Some(ref set) = include_set {
            if !set.is_match(&rel) {
                continue;
            }
        }

        // Apply exclude globs
        if let Some(ref set) = exclude_set {
            if set.is_match(&rel) {
                continue;
            }
        }

        // Skip binary files
        if is_binary(path) {
            continue;
        }

        match fs::read_to_string(path) {
            Ok(content) => {
                files.push(CollectedFile {
                    relative_path: rel,
                    content,
                });
            }
            Err(_) => continue,
        }
    }

    files.sort_by(|a, b| a.relative_path.cmp(&b.relative_path));
    files
}

fn format_markdown(files: &[CollectedFile], show_tokens: bool) -> String {
    let mut output = String::new();

    if show_tokens {
        let total_tokens: usize = files.iter().map(|f| estimate_tokens(&f.content)).sum();
        let total_bytes: usize = files.iter().map(|f| f.content.len()).sum();
        let _ = writeln!(
            output,
            "<!-- {files} files | {bytes} bytes | ~{tokens} tokens -->",
            files = files.len(),
            bytes = total_bytes,
            tokens = total_tokens
        );
        let _ = writeln!(output);
    }

    for file in files {
        let lang = lang_tag(Path::new(&file.relative_path));
        let _ = writeln!(output, "## `{}`", file.relative_path);
        let _ = writeln!(output);
        let _ = writeln!(output, "```{lang}");
        // Ensure content doesn't end with extra newlines inside fence
        let content = file.content.trim_end_matches('\n');
        let _ = writeln!(output, "{content}");
        let _ = writeln!(output, "```");
        let _ = writeln!(output);
    }

    output
}

fn format_xml(files: &[CollectedFile], show_tokens: bool) -> String {
    let mut output = String::new();

    let _ = writeln!(output, "<context>");

    if show_tokens {
        let total_tokens: usize = files.iter().map(|f| estimate_tokens(&f.content)).sum();
        let _ = writeln!(
            output,
            "  <meta files=\"{}\" tokens=\"~{}\"/>",
            files.len(),
            total_tokens
        );
    }

    for file in files {
        let _ = writeln!(output, "  <file path=\"{}\">", file.relative_path);
        // Simple XML escaping for content
        let escaped = file
            .content
            .replace('&', "&amp;")
            .replace('<', "&lt;")
            .replace('>', "&gt;");
        let _ = write!(output, "{escaped}");
        if !escaped.ends_with('\n') {
            let _ = writeln!(output);
        }
        let _ = writeln!(output, "  </file>");
    }

    let _ = writeln!(output, "</context>");

    output
}

fn print_tree(root: &Path, globs: &[String], excludes: &[String]) {
    let include_set = if globs.is_empty() {
        None
    } else {
        let mut builder = GlobSetBuilder::new();
        for g in globs {
            if let Ok(glob) = Glob::new(g) {
                builder.add(glob);
            }
        }
        builder.build().ok()
    };

    let exclude_set = if excludes.is_empty() {
        None
    } else {
        let mut builder = GlobSetBuilder::new();
        for g in excludes {
            if let Ok(glob) = Glob::new(g) {
                builder.add(glob);
            }
        }
        builder.build().ok()
    };

    let walker = WalkBuilder::new(root)
        .hidden(true)
        .git_ignore(true)
        .git_global(true)
        .git_exclude(true)
        .build();

    let mut paths: Vec<String> = Vec::new();

    for entry in walker.flatten() {
        let path = entry.path();
        if !path.is_file() {
            continue;
        }

        let rel = path
            .strip_prefix(root)
            .unwrap_or(path)
            .to_string_lossy()
            .to_string();

        if let Some(ref set) = include_set {
            if !set.is_match(&rel) {
                continue;
            }
        }
        if let Some(ref set) = exclude_set {
            if set.is_match(&rel) {
                continue;
            }
        }

        paths.push(rel);
    }

    paths.sort();

    // Print as a simple indented tree
    println!("{}/", root.file_name().unwrap_or(root.as_os_str()).to_string_lossy());
    for path_str in &paths {
        let parts: Vec<&str> = path_str.split('/').collect();
        let depth = parts.len() - 1;
        let indent = "  ".repeat(depth);
        let name = parts.last().unwrap_or(&"");
        println!("{indent}{name}");
    }

    println!("\n{} files", paths.len());
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::sync::atomic::{AtomicUsize, Ordering};

    static COUNTER: AtomicUsize = AtomicUsize::new(0);

    fn make_test_dir(name: &str) -> PathBuf {
        let id = COUNTER.fetch_add(1, Ordering::SeqCst);
        let dir = std::env::temp_dir().join(format!("gather_test_{name}_{id}"));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        fs::write(dir.join("hello.rs"), "fn main() {}\n").unwrap();
        fs::write(dir.join("data.json"), "{\"key\": \"value\"}\n").unwrap();
        fs::write(dir.join("notes.md"), "# Notes\n").unwrap();
        dir
    }

    #[test]
    fn test_estimate_tokens() {
        assert_eq!(estimate_tokens(""), 0);
        assert_eq!(estimate_tokens("hi"), 1); // 2 chars -> ceil(2/4) = 1
        assert_eq!(estimate_tokens("hello world"), 3); // 11 chars -> ceil(11/4) = 3
        assert_eq!(estimate_tokens("abcd"), 1); // exactly 4 chars -> 1 token
    }

    #[test]
    fn test_is_binary() {
        let dir = make_test_dir("is_binary");

        let text_file = dir.join("text.txt");
        fs::write(&text_file, "hello world").unwrap();
        assert!(!is_binary(&text_file));

        let bin_file = dir.join("binary.bin");
        fs::write(&bin_file, b"\x00\x01\x02\x03").unwrap();
        assert!(is_binary(&bin_file));

        fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn test_lang_tag() {
        assert_eq!(lang_tag(Path::new("main.rs")), "rust");
        assert_eq!(lang_tag(Path::new("app.py")), "python");
        assert_eq!(lang_tag(Path::new("index.js")), "javascript");
        assert_eq!(lang_tag(Path::new("config.toml")), "toml");
        assert_eq!(lang_tag(Path::new("Makefile")), "");
    }

    #[test]
    fn test_collect_files_basic() {
        let dir = make_test_dir("basic");
        let files = collect_files(&dir, &[], &[], 102400);
        assert_eq!(files.len(), 3);

        let paths: Vec<&str> = files.iter().map(|f| f.relative_path.as_str()).collect();
        assert!(paths.contains(&"hello.rs"));
        assert!(paths.contains(&"data.json"));
        assert!(paths.contains(&"notes.md"));

        fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn test_collect_files_glob_filter() {
        let dir = make_test_dir("glob");
        let globs = vec!["*.rs".to_string()];
        let files = collect_files(&dir, &globs, &[], 102400);
        assert_eq!(files.len(), 1);
        assert_eq!(files[0].relative_path, "hello.rs");

        fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn test_collect_files_exclude() {
        let dir = make_test_dir("exclude");
        let excludes = vec!["*.json".to_string()];
        let files = collect_files(&dir, &[], &excludes, 102400);
        let paths: Vec<&str> = files.iter().map(|f| f.relative_path.as_str()).collect();
        assert!(!paths.contains(&"data.json"));
        assert!(paths.contains(&"hello.rs"));

        fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn test_collect_files_max_size() {
        let dir = make_test_dir("maxsize");
        // Set max size to 5 bytes — should exclude most files
        let files = collect_files(&dir, &[], &[], 5);
        // All our test files are > 5 bytes
        assert!(files.is_empty());

        fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn test_format_markdown() {
        let files = vec![CollectedFile {
            relative_path: "test.rs".to_string(),
            content: "fn main() {}\n".to_string(),
        }];
        let output = format_markdown(&files, false);
        assert!(output.contains("## `test.rs`"));
        assert!(output.contains("```rust"));
        assert!(output.contains("fn main() {}"));
    }

    #[test]
    fn test_format_markdown_with_tokens() {
        let files = vec![CollectedFile {
            relative_path: "test.rs".to_string(),
            content: "fn main() {}\n".to_string(),
        }];
        let output = format_markdown(&files, true);
        assert!(output.contains("<!-- 1 files"));
        assert!(output.contains("tokens -->"));
    }

    #[test]
    fn test_format_xml() {
        let files = vec![CollectedFile {
            relative_path: "test.rs".to_string(),
            content: "fn main() {}\n".to_string(),
        }];
        let output = format_xml(&files, false);
        assert!(output.contains("<context>"));
        assert!(output.contains("<file path=\"test.rs\">"));
        assert!(output.contains("</context>"));
    }

    #[test]
    fn test_format_xml_escapes_special_chars() {
        let files = vec![CollectedFile {
            relative_path: "test.txt".to_string(),
            content: "a < b && c > d\n".to_string(),
        }];
        let output = format_xml(&files, false);
        assert!(output.contains("a &lt; b &amp;&amp; c &gt; d"));
    }
}

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Commands::Collect {
            path,
            globs,
            excludes,
            max_size,
            format,
            tokens,
        } => {
            let root = path.canonicalize().unwrap_or(path);
            let files = collect_files(&root, &globs, &excludes, max_size);

            if files.is_empty() {
                eprintln!("No files found matching the given criteria.");
                std::process::exit(1);
            }

            let output = match format {
                OutputFormat::Markdown => format_markdown(&files, tokens),
                OutputFormat::Xml => format_xml(&files, tokens),
            };

            print!("{output}");
        }

        Commands::Tree {
            path,
            globs,
            excludes,
        } => {
            let root = path.canonicalize().unwrap_or(path);
            print_tree(&root, &globs, &excludes);
        }

        Commands::Tokens {
            path,
            globs,
            excludes,
            max_size,
        } => {
            let root = path.canonicalize().unwrap_or(path);
            let files = collect_files(&root, &globs, &excludes, max_size);

            if files.is_empty() {
                eprintln!("No files found matching the given criteria.");
                std::process::exit(1);
            }

            let mut total_tokens = 0usize;
            let mut total_bytes = 0usize;

            for file in &files {
                let tokens = estimate_tokens(&file.content);
                let bytes = file.content.len();
                total_tokens += tokens;
                total_bytes += bytes;
                println!(
                    "{:>8} tokens  {:>8} bytes  {}",
                    tokens, bytes, file.relative_path
                );
            }

            println!();
            println!(
                "{:>8} tokens  {:>8} bytes  total ({} files)",
                total_tokens,
                total_bytes,
                files.len()
            );
        }
    }
}
