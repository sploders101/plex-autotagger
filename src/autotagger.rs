use std::{
	collections::HashMap,
	path::{Path, PathBuf},
};

use anyhow::Context;
use dialoguer::{Confirm, MultiSelect, Select};
use lazy_regex::regex;
use tmdb_async::{Episode, TV};
use tokio::{
	fs::{self, File},
	io::AsyncReadExt,
	sync::mpsc,
	task,
};
use urlencoding::encode;

use crate::{
	extract_subtitles::extract_subtitles,
	global_vars::TMDB_API_KEY,
	interact::{interact, interact_async},
	opensubtitles::get_subtitles,
	THEME,
};

pub async fn tag_items() -> anyhow::Result<()> {
	let mut episodes = HashMap::<u32, Episode>::from_iter(
		get_episodes_from_user()
			.await?
			.into_iter()
			.map(|episode| (episode.id, episode)),
	);

	let manually_select_subs = interact(|| {
		Confirm::with_theme(&*THEME)
			.with_prompt("Would you like to select subtitles manually?")
			.interact()
	})
	.await?;
	let mut subtitle_files = HashMap::<u32, String>::default();
	let mut missing_subtitles = Vec::<u32>::new();
	for episode in episodes.values() {
		let subtitles = get_subtitles(episode, manually_select_subs).await;
		match subtitles {
			Ok(subtitles) => {
				subtitle_files.insert(episode.id, strip_subtitles(&subtitles));
			}
			Err(_) => {
				missing_subtitles.push(episode.id);
				println!(
					"Skipping S{:02}E{:02}. No subtitles found.",
					episode.season_number, episode.episode_number
				);
			}
		}
	}
	for episode_id in missing_subtitles {
		episodes.remove(&episode_id);
	}

	// Get list of subtitle files without extensions and their contents
	let mut files = get_subtitle_files(".").await?;
	if files.len() == 0 {
		let new_files = interact_async(async {
			let should_extract = task::spawn_blocking(|| {
				Confirm::with_theme(&*THEME)
					.with_prompt("Subtitles not found. Would you like to extract them?")
					.interact()
			})
			.await??;
			if should_extract {
				extract_subtitles(false, None).await?;
				eprintln!("Got subtitles");
				let new_files = get_subtitle_files(".").await?;
				eprintln!("Listing new files");
				return Ok(Some(new_files));
			} else {
				return anyhow::Result::<Option<Vec<(PathBuf, String)>>>::Ok(None);
			}
		})
		.await?;
		match new_files {
			Some(new_files) => files = new_files,
			None => {
				println!("Cannot continue without subtitles.");
				return Ok(());
			}
		}
	}

	// Order potential matches by likeness
	// eprintln!("Running levenshtein distances...");
	let mut matches = Vec::<(u32, usize, &PathBuf)>::new();
	rayon::scope(|s| {
		let (lev_sender, mut lev_receiver) = mpsc::unbounded_channel::<(u32, usize, &PathBuf)>();
		for episode in episodes.values() {
			for (file, contents) in &files {
				// let episode_id = episode.id;
				let lev_sender = lev_sender.clone();
				let subtitle_files = &subtitle_files;
				s.spawn(move |_| {
					let distance =
						strsim::levenshtein(subtitle_files.get(&episode.id).unwrap(), contents);
					lev_sender.send((episode.id, distance, file)).unwrap();
				});
			}
		}
		drop(lev_sender);
		while let Some(entry) = lev_receiver.blocking_recv() {
			// eprintln!("Finished distance for episode {} against {}: {}", entry.0, entry.2.file_name().unwrap().to_str().unwrap(), entry.1);
			matches.push(entry);
		}
	});
	matches.sort_unstable();
	// eprintln!("Finished levelshtein distances");

	// Match against the known files first to eliminate extras and such.
	let mut matches_by_episode = HashMap::<u32, Vec<(&PathBuf, usize)>>::new();
	for (episode_id, likeness, path) in matches {
		match matches_by_episode.get_mut(&episode_id) {
			Some(results) => {
				results.push((path, likeness));
			}
			None => {
				matches_by_episode.insert(episode_id, vec![(path, likeness)]);
			}
		}
	}

	let mut matches_by_file = HashMap::<&PathBuf, Vec<(usize, &Episode)>>::new();
	for (episode_id, matches) in matches_by_episode {
		if matches.len() == 0 {
			continue;
		}
		for (file_path, distance) in matches {
			match matches_by_file.get_mut(file_path) {
				Some(file_matches) => {
					file_matches.push((distance, episodes.get(&episode_id).unwrap()));
				}
				None => {
					matches_by_file.insert(
						file_path,
						vec![(distance, episodes.get(&episode_id).unwrap())],
					);
				}
			}
		}
	}

	for (file_path, matches) in matches_by_file.iter_mut() {
		matches.sort_unstable_by_key(|sort| sort.0);

		let mkv_file = file_path.with_extension("mkv");
		let srt_file = file_path.with_extension("srt");

		let mut rename_to = match matches.len() {
			0 => {
				println!("{:?} => ??? (No match found)", *file_path);
				None
			}
			1 => {
				let filename = format_filename(matches[0].1);
				println!(
					"{:?} => {:?}\n    distance:         {}\n    closest_negative: n/a",
					&mkv_file, &filename, matches[0].0
				);
				Some(filename)
			}
			_ => {
				let filename = format_filename(matches[0].1);
				println!(
					"{:?} => {:?}\n    distance:         {}\n    closest negative: {}",
					&mkv_file, &filename, matches[0].0, matches[1].0
				);
				Some(filename)
			}
		};

		let rename = interact(|| {
			Confirm::with_theme(&*THEME)
				.with_prompt("Rename file?")
				.interact()
		})
		.await?;
		if !rename {
			rename_to = None;
		}
		if let Some(rename_to) = rename_to {
			fs::rename(mkv_file, rename_to)
				.await
				.context("Couldn't rename mkv file")?;
			fs::remove_file(srt_file)
				.await
				.context("Failed to remove srt file")?;
		}
	}

	return Ok(());
}

fn format_filename(episode: &Episode) -> PathBuf {
	return PathBuf::from(format!(
		"S{:02}E{:02} - {}.mkv",
		episode.season_number, episode.episode_number, episode.name
	));
}

pub async fn get_tv_show(tmdb_client: &tmdb_async::Client) -> anyhow::Result<TV> {
	// Ask the user for search query
	let input_title: String = interact(|| {
		dialoguer::Input::with_theme(&*THEME)
			.with_prompt("Title")
			.interact_text()
	})
	.await?;
	let titles = tmdb_client.tv_search(&encode(&input_title), None).await?;

	// Ask the user which search result to use
	let title_descriptions: Vec<String> = titles
		.results()
		.iter()
		.map(|title| {
			match title.first_air_date() {
				Some(first_air_date) => format!(
					"{}: {} ({})",
					title.id(),
					title.name(),
					first_air_date
				),
				None => format!(
					"{}: {}",
					title.id(),
					title.name(),
				),
			}
		})
		.collect();
	let selected_title_index: usize = interact(move || {
		Select::with_theme(&*THEME)
			.items(&title_descriptions)
			.with_prompt("Please select the desired result")
			.default(0)
			.interact()
	})
	.await?;
	return Ok(tmdb_client
		.tv_by_id(titles.results()[selected_title_index].id(), false, false)
		.await
		.context("Couldn't get TV show")?);
}

pub async fn get_episodes_from_user() -> anyhow::Result<Vec<Episode>> {
	let tmdb_client = tmdb_async::Client::new(TMDB_API_KEY.clone());

	let selected_title = get_tv_show(&tmdb_client).await?;

	// Get list of desired seasons from user
	let season_names: Vec<String> = selected_title
		.seasons()
		.iter()
		.map(|season| format!("{} ({} episodes)", season.name(), season.episode_count()))
		.collect();
	let desired_seasons_idx = interact(move || {
		MultiSelect::with_theme(&*THEME)
			.items(&season_names)
			.with_prompt("Please select the seasons included on this disc")
			.interact()
	})
	.await?;
	let mut desired_seasons = Vec::<_>::new();
	for idx in desired_seasons_idx {
		desired_seasons.push(
			tmdb_client
				.season(
					selected_title.id(),
					selected_title.seasons()[idx].season_number(),
				)
				.await?,
		);
	}

	// Get list of desired episodes from the user, organized by season
	let mut desired_episodes = Vec::<Episode>::new();
	for season in desired_seasons {
		let episode_names: Vec<String> = season
			.episodes
			.iter()
			.map(|item| format!("Episode {} - {}", item.episode_number, item.name))
			.collect();
		let desired_episode_indexes = interact(move || {
			MultiSelect::with_theme(&*THEME)
				.items(&episode_names)
				.with_prompt(format!(
					"Please select the episodes included on this disc from {}",
					season.name
				))
				.interact()
		})
		.await?;
		desired_episodes.extend(
			desired_episode_indexes
				.into_iter()
				.map(|index| season.episodes[index].clone()),
		);
	}

	return Ok(desired_episodes);
}

async fn get_subtitle_files(location: impl AsRef<Path>) -> anyhow::Result<Vec<(PathBuf, String)>> {
	let mut files_iter = fs::read_dir(location).await?;
	let mut files = Vec::<(PathBuf, String)>::new();
	while let Some(file) = files_iter.next_entry().await? {
		let path = file.path();
		if let Some(Some("srt")) = path.extension().map(|ext| ext.to_str()) {
			let mut contents = String::new();
			File::open(&path)
				.await?
				.read_to_string(&mut contents)
				.await?;
			files.push((path.with_extension(""), strip_subtitles(&contents)));
		}
	}
	return Ok(files);
}

/// Strips symbols from subtitles that may cause issues during comparison
pub fn strip_subtitles(subs: &str) -> String {
	let intermediate = regex!(
		r"(?:<\s*[^>]*>|<\s*/\s*a>)|(?:^.*-->.*$|^[0-9]+$|[^a-zA-Z0-9 ?\.,!\n]|^\s*-*\s*|\r)"m
	)
	.replace_all(subs, "");
	return regex!(r"[\n ]{1,}")
		.replace_all(&intermediate, " ")
		.into_owned();
}
