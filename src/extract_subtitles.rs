use anyhow::anyhow;
use crate::get_st_track::get_comparison_track;
use std::path::PathBuf;
use crate::task_queue::TaskQueue;
use tokio::{fs, process::Command};

pub async fn extract_subtitles(skip_ocr: bool, files: Vec<PathBuf>) -> anyhow::Result<()> {
	let ocr_queue = if skip_ocr {
		None
	} else {
		Some(TaskQueue::new())
	};

	for file in files {
		let st_track = get_comparison_track(&file).await?;
		let extract_result = Command::new("mkvextract")
			.arg("tracks")
			.arg(&file)
			.arg(format!(
				"{}:{}",
				st_track.number - 1,
				file.with_extension("").to_str().unwrap()
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
			ocr_queue
				.add_task(async move {
					if st_track.codec_id == "S_VOBSUB" {
						let ocr_result = Command::new("vobsubocr")
							.args([
								"-c",
								"tessedit_char_blacklist=|\\/`_~",
								"-l",
								"eng",
								"-o",
							])
							.arg(file.with_extension("srt"))
							.arg(file.with_extension("idx"))
							.spawn()
							.unwrap()
							.wait()
							.await
							.unwrap();
						if ocr_result.success() {
							let _ = fs::remove_file(file.with_extension("idx")).await;
							let _ = fs::remove_file(file.with_extension("sub")).await;
						}
					}
				})
				.await;
		}
	}

	if let Some(ocr_queue) = ocr_queue {
		ocr_queue.wait_for_queued_tasks().await;
	}

	return Ok(());
}
