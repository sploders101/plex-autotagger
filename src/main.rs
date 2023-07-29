mod extract_subtitles;
mod get_st_track;
mod interact;
mod task_queue;
mod autotagger;
mod opensubtitles;

use autotagger::tag_items;
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

		#[arg()]
		files: Vec<PathBuf>,
	},

	/// Scans subtitle files to identify requested episodes by way of subtitle comparison
	Tag,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
	let args = Cli::parse();

	match args.command {
		AutotaggerCommand::ExtractSubtitles { skip_ocr, files } => {
			if files.len() == 0 {
				extract_subtitles(skip_ocr, None).await?;
			} else {
				extract_subtitles(skip_ocr, Some(files)).await?;
			}
		}
		AutotaggerCommand::Tag => {
			tag_items().await?;
		}
	}

	return Ok(());
}

