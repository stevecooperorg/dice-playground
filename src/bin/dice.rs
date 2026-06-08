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
        } => run_eval(&path, format, prob_format),
        Command::Table2d10 => run_table_2d10(),
        Command::Docs { out } => run_docs(out.as_deref()),
        Command::Lsp => dice_playground::engine::lsp::run_stdio(),
    }
}

fn run_docs(out: Option<&std::path::Path>) -> anyhow::Result<()> {
    let md = render_stdlib_reference_markdown();
    match out {
        Some(path) => fs::write(path, &md).with_context(|| format!("write {}", path.display()))?,
        None => print!("{md}"),
    }
    Ok(())
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
