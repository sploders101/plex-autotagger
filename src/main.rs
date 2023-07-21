mod get_st_track;
mod interact;

use anyhow::anyhow;
use clap::{Parser, Subcommand};
use lazy_static::lazy_static;
use std::{path::PathBuf, process::Command};
use get_st_track::get_comparison_track;

lazy_static! {
	static ref THEME: dialoguer::theme::ColorfulTheme = {
		dialoguer::theme::ColorfulTheme::default()
	};
}

#[derive(Parser)]
struct Cli {
	#[command(subcommand)]
	command: AutotaggerCommand,
}

#[derive(Subcommand)]
enum AutotaggerCommand {
	ExtractSubtitles {
	    files: Vec<PathBuf>,
	}
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = Cli::parse();

	match args.command {
		AutotaggerCommand::ExtractSubtitles { files } => {
			for file in files {
				let st_track = get_comparison_track(&file).await?;
				let extract_result = Command::new("mkvextract")
					.arg("tracks")
					.arg(&file)
					.arg(format!("{}:{}", st_track.number - 1, file.with_extension("").to_str().unwrap()))
					.spawn().unwrap()
					.wait().unwrap();
				if !extract_result.success() {
					return Err(anyhow!("Failed to extract subtitles"));
				}
			}
		}
	}

    return Ok(());
}
