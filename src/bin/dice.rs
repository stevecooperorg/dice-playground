use std::fs;
use std::path::PathBuf;

use anyhow::Context;
use clap::{Parser, ValueEnum};
use dice_playground::engine::{
    desugar_if_needed, eval_source, format_eval_result_text, render_stdlib_reference_markdown,
    DieRoll, OutputEntry, ProbFormat,
};

#[derive(Parser)]
#[command(
    name = "dice",
    about = "Evaluate dice probability scripts (Starlark + dice stdlib)"
)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(clap::Subcommand)]
enum Command {
    /// Evaluate a `.dice` script (Starlark + tabletop dice sugar).
    Eval {
        /// Script path.
        path: PathBuf,
        #[arg(long, value_enum, default_value_t = Format::Text)]
        format: Format,
        /// How to print probabilities in text output (decimal, percent, or fraction).
        #[arg(long, value_enum, default_value_t = ProbFormat::Decimal)]
        prob_format: ProbFormat,
        /// Re-run evaluation whenever the script file changes (runs once immediately).
        #[arg(long)]
        watch: bool,
    },
    /// Print a 2d10 + modifier success table (CSV).
    Table2d10,
    /// Print Markdown function reference for dice builtins (from Rust doc comments).
    Docs {
        /// Write Markdown to this file instead of stdout.
        #[arg(long)]
        out: Option<PathBuf>,
    },
    /// Language server (stdio LSP).
    Lsp,
    /// Inject playground load links into static HTML under tutorial/, cookbook/, docs/, references/.
    EnhanceStaticSite {
        /// Site output directory (e.g. dist/ or static-site/).
        #[arg(default_value = "dist")]
        root: PathBuf,
    },
}

#[derive(Clone, Copy, ValueEnum)]
enum Format {
    Text,
    Json,
    Csv,
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    match cli.command {
        Command::Eval {
            path,
            format,
            prob_format,
            watch,
        } => {
            if watch {
                run_eval_watch(&path, format, prob_format)
            } else {
                run_eval(&path, format, prob_format)
            }
        }
        Command::Table2d10 => run_table_2d10(),
        Command::Docs { out } => run_docs(out.as_deref()),
        Command::Lsp => dice_playground::engine::lsp::run_stdio(),
        Command::EnhanceStaticSite { root } => run_enhance_static_site(&root),
    }
}

fn run_enhance_static_site(root: &std::path::Path) -> anyhow::Result<()> {
    let n = dice_playground::ui::static_site::enhance_static_site_tree(root)?;
    eprintln!("enhanced {n} HTML file(s) under {}", root.display());
    Ok(())
}

fn run_docs(out: Option<&std::path::Path>) -> anyhow::Result<()> {
    let md = render_stdlib_reference_markdown();
    match out {
        Some(path) => fs::write(path, &md).with_context(|| format!("write {}", path.display()))?,
        None => print!("{md}"),
    }
    Ok(())
}

fn run_eval_watch(
    path: &std::path::Path,
    format: Format,
    prob_format: ProbFormat,
) -> anyhow::Result<()> {
    use dice_playground::cli::file_watcher::FileWatcher;

    let watch_root = path
        .parent()
        .map(|p| p.to_path_buf())
        .filter(|p| !p.as_os_str().is_empty())
        .unwrap_or_else(|| PathBuf::from("."));

    let mut watcher = FileWatcher::new(watch_root)?;
    watcher.watch_path(path, false)?;

    try_run_eval(path, format, prob_format);

    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .context("create tokio runtime for watch mode")?;

    runtime.block_on(async {
        loop {
            watcher
                .wait_for_change_matching(|changed| paths_same_file(changed, path))
                .await?;
            // Coalesce rapid save/rename events from editors.
            tokio::time::sleep(std::time::Duration::from_millis(100)).await;
            eprintln!("--- {} changed, re-evaluating ---", path.display());
            try_run_eval(path, format, prob_format);
        }
    })
}

/// Run evaluation; on failure print to stderr and keep going (watch mode).
fn try_run_eval(path: &std::path::Path, format: Format, prob_format: ProbFormat) {
    if let Err(e) = run_eval(path, format, prob_format) {
        eprintln!("{e:#}");
    }
}

fn paths_same_file(a: &std::path::Path, b: &std::path::Path) -> bool {
    match (a.canonicalize(), b.canonicalize()) {
        (Ok(a), Ok(b)) => a == b,
        _ => a == b,
    }
}

fn run_eval(path: &std::path::Path, format: Format, prob_format: ProbFormat) -> anyhow::Result<()> {
    let content = fs::read_to_string(path).with_context(|| format!("read {}", path.display()))?;
    let path_str = path.to_string_lossy();
    let expanded = desugar_if_needed(&path_str, &content)?;
    let result = eval_source(&path_str, &expanded)?;
    match format {
        Format::Text => print!("{}", format_eval_result_text(&result, prob_format)),
        Format::Json => println!("{}", serde_json::to_string_pretty(&result.outputs)?),
        Format::Csv => print_outputs_csv(&result.outputs)?,
    }
    Ok(())
}

fn print_outputs_csv(outputs: &[OutputEntry]) -> anyhow::Result<()> {
    let mut wtr = csv::Writer::from_writer(vec![]);
    for out in outputs {
        match out {
            OutputEntry::DieRoll {
                name,
                entries,
                mean,
            } => {
                for (value, prob) in entries {
                    wtr.write_record([
                        name.as_str(),
                        "dieroll",
                        &value.to_string(),
                        &prob.to_string(),
                    ])?;
                }
                wtr.write_record([name.as_str(), "mean", "", &mean.to_string()])?;
            }
            OutputEntry::Prob { name, value } => {
                wtr.write_record([name.as_str(), "prob", "", &value.to_string()])?;
            }
            OutputEntry::Outcomes {
                name,
                entries,
                scale: _,
            } => {
                for (label, prob) in entries {
                    wtr.write_record([
                        name.as_str(),
                        "outcomes",
                        label.as_str(),
                        &prob.to_string(),
                    ])?;
                }
            }
            OutputEntry::Table { name, entries } => {
                for (label, prob) in entries {
                    wtr.write_record([name.as_str(), "table", label.as_str(), &prob.to_string()])?;
                }
            }
        }
    }
    let bytes = wtr.into_inner()?;
    print!("{}", String::from_utf8(bytes)?);
    Ok(())
}

fn run_table_2d10() -> anyhow::Result<()> {
    let d10 = DieRoll::die(10)?;
    let roll = d10.convolve(&d10)?;
    let mut wtr = csv::Writer::from_writer(vec![]);
    wtr.write_record(["target", "modifier", "p_success_pct"])?;
    for target in 0..=10 {
        for modifier in 0..=9 {
            let shifted = roll.shift(modifier)?;
            let effective = (target - modifier).max(0);
            let p = shifted.p_ge(effective);
            wtr.write_record([
                target.to_string(),
                modifier.to_string(),
                format!("{:.0}", p * 100.0),
            ])?;
        }
    }
    let bytes = wtr.into_inner()?;
    print!("{}", String::from_utf8(bytes)?);
    Ok(())
}
