//! karp - Aozora Bunko text to EPUB compiler
//!
//! Usage:
//!   karp build <path>  - Compile text file to EPUB
//!   karp check <path>  - Check for warnings/errors without generating EPUB

use aozora_parser::{
    parse_aozora, parse, parse_blocks, lint, text_to_epub,
    LintWarning, Severity, ConversionError,
};
use clap::{Parser, Subcommand};
use encoding_rs::SHIFT_JIS;
use std::fs;
use std::path::PathBuf;
use std::process::ExitCode;

#[derive(Parser)]
#[command(name = "karp")]
#[command(author, version, about = "Aozora Bunko text to EPUB compiler")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Compile text file to EPUB
    Build {
        /// Path to the input text file
        path: PathBuf,
    },
    /// Check for warnings/errors without generating EPUB
    Check {
        /// Path to the input text file
        path: PathBuf,
    },
}

fn main() -> ExitCode {
    let cli = Cli::parse();

    match cli.command {
        Commands::Build { path } => build_command(&path),
        Commands::Check { path } => check_command(&path),
    }
}

fn build_command(path: &PathBuf) -> ExitCode {
    println!("   \x1b[1;32mCompiling\x1b[0m {}", path.display());

    // Read and decode file
    let text = match read_aozora_file(path) {
        Ok(t) => t,
        Err(e) => {
            print_error(&format!("could not read file: {}", e));
            return ExitCode::FAILURE;
        }
    };

    // Run linter and collect warnings
    let warnings = match run_lint(&text) {
        Ok(w) => w,
        Err(e) => {
            print_conversion_error(&e, path);
            return ExitCode::FAILURE;
        }
    };

    // Print warnings
    let error_count = print_warnings(&warnings, path);

    if error_count > 0 {
        print_summary(error_count, warnings.len() - error_count, true);
        return ExitCode::FAILURE;
    }

    // Generate EPUB
    let output_path = path.with_extension("epub");
    match text_to_epub(text, &output_path) {
        Ok(()) => {
            if !warnings.is_empty() {
                print_summary(0, warnings.len(), false);
            }
            println!("    \x1b[1;32mFinished\x1b[0m {}", output_path.display());
            ExitCode::SUCCESS
        }
        Err(e) => {
            print_conversion_error(&e, path);
            ExitCode::FAILURE
        }
    }
}

fn check_command(path: &PathBuf) -> ExitCode {
    println!("    \x1b[1;32mChecking\x1b[0m {}", path.display());

    // Read and decode file
    let text = match read_aozora_file(path) {
        Ok(t) => t,
        Err(e) => {
            print_error(&format!("could not read file: {}", e));
            return ExitCode::FAILURE;
        }
    };

    // Run linter and collect warnings
    let warnings = match run_lint(&text) {
        Ok(w) => w,
        Err(e) => {
            print_conversion_error(&e, path);
            return ExitCode::FAILURE;
        }
    };

    // Print warnings
    let error_count = print_warnings(&warnings, path);
    print_summary(error_count, warnings.len() - error_count, error_count > 0);

    if error_count > 0 {
        ExitCode::FAILURE
    } else {
        ExitCode::SUCCESS
    }
}

fn read_aozora_file(path: &PathBuf) -> Result<String, String> {
    let bytes = fs::read(path).map_err(|e| e.to_string())?;
    
    // Try Shift_JIS first, then fall back to UTF-8
    let (cow, _, had_errors) = SHIFT_JIS.decode(&bytes);
    if had_errors {
        // Try UTF-8
        String::from_utf8(bytes.clone())
            .map_or_else(|_| Ok(cow.into_owned()), Ok)
    } else {
        Ok(cow.into_owned())
    }
}

fn run_lint(text: &str) -> Result<Vec<LintWarning>, ConversionError> {
    let tokens = parse_aozora(text.to_string())?;
    let doc = parse(tokens)?;
    let blocks = parse_blocks(doc.items)?;
    let result = lint(blocks, text);
    Ok(result.warnings)
}

fn print_warnings(warnings: &[LintWarning], path: &PathBuf) -> usize {
    let mut error_count = 0;

    for w in warnings {
        let (color, label) = match w.severity {
            Severity::Error => {
                error_count += 1;
                ("\x1b[1;31m", "error")
            }
            Severity::Warning => ("\x1b[1;33m", "warning"),
            Severity::Info => ("\x1b[1;36m", "info"),
        };

        println!(
            "{}{}\x1b[0m: {}",
            color, label, w.message
        );
        println!(
            "  \x1b[1;34m-->\x1b[0m {}:{}",
            path.display(),
            w.span.start
        );
        println!();
    }

    error_count
}

fn print_summary(errors: usize, warnings: usize, is_error: bool) {
    if is_error {
        print!("\x1b[1;31merror\x1b[0m: ");
        println!(
            "aborting due to {} error{}{}",
            errors,
            if errors == 1 { "" } else { "s" },
            if warnings > 0 {
                format!("; {} warning{} emitted", warnings, if warnings == 1 { "" } else { "s" })
            } else {
                String::new()
            }
        );
    } else if warnings > 0 {
        println!(
            "\x1b[1;33mwarning\x1b[0m: {} warning{} emitted",
            warnings,
            if warnings == 1 { "" } else { "s" }
        );
    }
}

fn print_error(msg: &str) {
    println!("\x1b[1;31merror\x1b[0m: {}", msg);
}

fn print_conversion_error(e: &ConversionError, path: &PathBuf) {
    println!("\x1b[1;31merror\x1b[0m: {}", e);
    println!("  \x1b[1;34m-->\x1b[0m {}", path.display());
}
