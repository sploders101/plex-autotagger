mod extract_subtitles;
mod get_st_track;
mod interact;
mod task_queue;

use clap::{Parser, Subcommand};
use extract_subtitles::extract_subtitles;
use lazy_static::lazy_static;
use std::path::PathBuf;

lazy_static! {
	static ref THEME: dialoguer::theme::ColorfulTheme = dialoguer::theme::ColorfulTheme::default();
}

#[derive(Parser)]
struct Cli {
	#[command(subcommand)]
	command: AutotaggerCommand,
}

#[derive(Subcommand)]
enum AutotaggerCommand {
	/// Extracts subtitles from a set of mkv files, generating srt files.
	ExtractSubtitles {
		/// Skips running OCR on bitmap-style subtitles, leaving them in sub/idx format
		#[arg(short, long)]
		skip_ocr: bool,

		#[arg(required = true)]
		files: Vec<PathBuf>,
	},
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
	let args = Cli::parse();

	match args.command {
		AutotaggerCommand::ExtractSubtitles { skip_ocr, files } => {
			extract_subtitles(skip_ocr, files).await?;
		}
	}

	return Ok(());
}
