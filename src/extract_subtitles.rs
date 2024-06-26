use crate::task_queue::TaskQueue;
use crate::{get_st_track::get_comparison_track, interact::interact, THEME};
use anyhow::anyhow;
use std::path::PathBuf;
use std::process::Stdio;
use std::time::Duration;
use tokio::{fs, process::Command};

pub async fn extract_subtitles(skip_ocr: bool, files: Option<Vec<PathBuf>>) -> anyhow::Result<()> {
	let ocr_queue = if skip_ocr {
		None
	} else {
		Some(TaskQueue::new())
	};

	let files = match files {
		Some(files) => files,
		None => {
			let mut listing = fs::read_dir(std::env::current_dir()?).await?;
			let mut files = Vec::<PathBuf>::new();
			while let Some(item) = listing.next_entry().await? {
				let path = item.path();
				if item.file_type().await?.is_file()
					&& path
						.extension()
						.is_some_and(|inner| inner.to_str().is_some_and(|inner| inner == "mkv"))
				{
					files.push(path);
				}
			}
			files
		}
	};

	for file in files {
		let st_track = match get_comparison_track(&file).await? {
			Some(track) => track,
			None => {
				println!(
					"No suitable subtitles found for {}",
					file.into_os_string()
						.into_string()
						.expect("Invalid file name")
				);
				continue;
			}
		};
		let track_file = file.with_extension(match st_track.codec_id.as_str() {
			"S_TEXT/UTF8" => "srt",
			"S_VOBSUB" => "sub",
			"S_HDMV/PGS" => "sup",
			_ => "dat",
		});
		let extract_result = Command::new("mkvextract")
			.arg("tracks")
			.arg(&file)
			.arg(format!(
				"{}:{}",
				st_track.number - 1,
				track_file.to_str().unwrap()
			))
			.spawn()
			.unwrap()
			.wait()
			.await
			.unwrap();
		if !extract_result.success() {
			return Err(anyhow!("Failed to extract subtitles"));
		}

		if let Some(ref ocr_queue) = ocr_queue {
			ocr_queue.add_task(async move {
				let mut vobsubocr = false;
				match st_track.codec_id.as_str() {
					"S_VOBSUB" => vobsubocr = true,
					"S_HDMV/PGS" => {
						let bdsup_path = match std::env::var("BDSUP2SUB_PATH") {
							Ok(path) => path,
							Err(_) => interact(|| {
								dialoguer::Input::with_theme(&*THEME)
									.with_prompt("Path to BDSup2Sub.jar")
									.interact_text()
							})
							.await
							.unwrap(),
						};
						let bdsup_result = Command::new("java")
							.args(["-jar", &bdsup_path, "-o"])
							.arg(file.with_extension("sub"))
							.arg(file.with_extension("sup"))
							.stdin(Stdio::null())
							.stdout(Stdio::null())
							.spawn()
							.unwrap()
							.wait()
							.await
							.unwrap();

						// This program seems to exit before it's actually finished. May need to do some bugfixing...
						// if bdsup_result.success() {}
						tokio::time::sleep(Duration::from_secs(1)).await;
						let sup_file = file.with_extension("sup");
						if let Err(err) = fs::remove_file(&sup_file).await {
							println!(
								"Could not delete {}. Error:\n{}",
								sup_file
									.file_name()
									.and_then(|inner| inner.to_str())
									.unwrap_or("unknown file"),
								err
							)
						}
						vobsubocr = true;
					}
					_ => {}
				}
				if vobsubocr {
					let ocr_result = Command::new("vobsubocr")
						.args(["-c", "tessedit_char_blacklist=|\\/`_~!", "-l", "eng", "-o"])
						.arg(file.with_extension("srt"))
						.arg(file.with_extension("idx"))
						.spawn()
						.unwrap()
						.wait()
						.await
						.unwrap();
					if ocr_result.success() {
						// Remove raster subtitle files
						let idx_file = file.with_extension("idx");
						if let Err(err) = fs::remove_file(&idx_file).await {
							println!(
								"Could not delete {}. Error:\n{}",
								idx_file
									.file_name()
									.and_then(|inner| inner.to_str())
									.unwrap_or("unknown file"),
								err
							);
						}
						let sub_file = file.with_extension("sub");
						if let Err(err) = fs::remove_file(&sub_file).await {
							println!(
								"Could not delete {}. Error:\n{}",
								sub_file
									.file_name()
									.and_then(|inner| inner.to_str())
									.unwrap_or("unknown file"),
								err
							);
						}
					}
				}
			});
		}
	}

	if let Some(ocr_queue) = ocr_queue {
		ocr_queue.wait_for_queued_tasks().await;
	}

	return Ok(());
}
