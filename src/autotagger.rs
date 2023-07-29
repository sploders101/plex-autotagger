use std::{sync::Arc, collections::HashMap, path::{PathBuf, Path}};

use anyhow::Context;
use dialoguer::{Select, MultiSelect, Confirm};
use lazy_regex::regex;
use lazy_static::lazy_static;
use tmdb_async::Episode;
use tokio::{sync::{RwLock, mpsc}, fs::{self, File}, io::AsyncReadExt, task};
use urlencoding::encode;

use crate::{interact::{interact, interact_async}, THEME, opensubtitles::get_subtitles, extract_subtitles::extract_subtitles};

lazy_static! {
	static ref TMDB_API_KEY: String = std::env::var("TMDB_API_KEY")
		.expect("API key not found. Please specify it with TMDB_API_KEY environment variable");
	static ref OST_API_KEY: RwLock<Option<Arc<str>>> = RwLock::new(None);
	static ref HTTP_CLIENT: reqwest::Client = reqwest::Client::new();
}

pub async fn tag_items() -> anyhow::Result<()> {
	let episodes = HashMap::<u32, Episode>::from_iter(
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
	for episode in episodes.values() {
		subtitle_files.insert(
			episode.id,
			strip_subtitles(&get_subtitles(episode, manually_select_subs).await?),
		);
		// eprintln!("Got subtitles for {:?}", &episode);
	}

	// Get list of subtitle files without extensions and their contents
	let mut files = get_subtitle_files(".").await?;
	if files.len() == 0 {
		let new_files = interact_async(async {
			let should_extract = task::spawn_blocking(|| {
				Confirm::with_theme(&*THEME)
					.with_prompt("Subtitles not found. Would you like to extract them?")
					.interact()
			}).await??;
			if should_extract {
				extract_subtitles(false, None).await?;
				eprintln!("Got subtitles");
				let new_files = get_subtitle_files(".").await?;
				eprintln!("Listing new files");
				return Ok(Some(new_files));
			} else {
				return anyhow::Result::<Option<Vec<(PathBuf, String)>>>::Ok(None);
			}
		}).await?;
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
					let distance = strsim::levenshtein(subtitle_files.get(&episode.id).unwrap(), contents);
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
			},
			None => {
				matches_by_episode.insert(episode_id, vec![(path, likeness)]);
			},
		}
	}

	let mut matches_by_file = HashMap::<&PathBuf, Vec<(usize, u32)>>::new();
	for (episode_id, matches) in matches_by_episode {
		if matches.len() == 0 {
			continue;
		}

		let episode = episodes.get(&episode_id).unwrap();
		println!(
			"S{}E{}: {} (differences: {}, closest negative match: {:?})",
			episode.season_number,
			episode.episode_number,
			matches[0].0.file_name().unwrap().to_str().unwrap(),
			matches[0].1,
			matches.get(1).map(|inner| inner.1),
		);
	}

	return Ok(());
}

pub async fn get_episodes_from_user() -> anyhow::Result<Vec<Episode>> {
	let tmdb_client = tmdb_async::Client::new(TMDB_API_KEY.clone());

	// Ask the user for search query
	let input_title: String =
		interact(|| dialoguer::Input::with_theme(&*THEME).with_prompt("Title").interact_text()).await?;
	let titles = tmdb_client.tv_search(&encode(&input_title), None).await?;

	// Ask the user which search result to use
	let title_descriptions: Vec<String> = titles
		.results()
		.iter()
		.map(|title| {
			format!(
				"{}: {} ({})",
				title.id(),
				title.name(),
				title.first_air_date()
			)
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
	let selected_title = tmdb_client.tv_by_id(
		titles.results()[selected_title_index].id(),
		false,
		false,
	).await.context("Couldn't get TV show")?;

	// Get list of desired seasons from user
	let season_names: Vec<String> = selected_title.seasons()
		.iter()
		.map(|season| format!("{} ({} episodes)", season.name(), season.episode_count()))
		.collect();
	let desired_seasons_idx = interact(move || {
		MultiSelect::with_theme(&*THEME)
			.items(&season_names)
			.with_prompt("Please select the seasons included on this disc")
			.interact()
	}).await?;
	let mut desired_seasons = Vec::<_>::new();
	for idx in desired_seasons_idx {
		desired_seasons.push(tmdb_client.season(selected_title.id(), selected_title.seasons()[idx].season_number()).await?);
	}

	// Get list of desired episodes from the user, organized by season
	let mut desired_episodes = Vec::<Episode>::new();
	for season in desired_seasons {
		let episode_names: Vec<String> = season.episodes
			.iter()
			.map(|item| format!("Episode {} - {}", item.episode_number, item.name))
			.collect();
		let desired_episode_indexes = interact(move || {
			MultiSelect::with_theme(&*THEME)
				.items(&episode_names)
				.with_prompt(format!("Please select the episodes included on this disc from {}", season.name))
				.interact()
		}).await?;
		desired_episodes.extend(desired_episode_indexes.into_iter().map(|index| season.episodes[index].clone()));
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
			File::open(&path).await?.read_to_string(&mut contents).await?;
			files.push((path.with_extension(""), strip_subtitles(&contents)));
		}
	}
	return Ok(files);
}

/// Strips symbols from subtitles that may cause issues during comparison
pub fn strip_subtitles(subs: &str) -> String {
	let intermediate = regex!(r"(?:<\s*[^>]*>|<\s*/\s*a>)|(?:^.*-->.*$|^[0-9]+$|[^a-zA-Z0-9 ?\.,!\n]|^\s*-*\s*|\r)"m)
		.replace_all(subs, "");
	return regex!(r"[\n ]{1,}").replace_all(&intermediate, " ").into_owned();
}
