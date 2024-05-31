use std::{sync::Arc, process::Stdio};

use anyhow::{Context, anyhow};
use async_trait::async_trait;
use dialoguer::{Input, Password, Select, Confirm};
use lazy_static::lazy_static;
use serde::Deserialize;
use serde_json::json;
use tmdb_async::Episode;
use tokio::{sync::RwLock, process::Command, io::AsyncWriteExt, task};

use crate::{interact::{interact, interact_async}, THEME};

lazy_static! {
	static ref TMDB_API_KEY: String = std::env::var("TMDB_API_KEY")
		.expect("API key not found. Please specify it with TMDB_API_KEY environment variable");
	static ref OST_API_KEY: String = std::env::var("OST_API_KEY")
		.expect("API key not found. Please specify it with OST_API_KEY environment variable");
	static ref OST_AUTH: RwLock<Option<Arc<str>>> = RwLock::new(None);
	static ref HTTP_CLIENT: reqwest::Client = reqwest::Client::new();
}

#[derive(Deserialize)]
struct LoginResponse {
	token: String,
}

pub async fn get_ost_auth() -> anyhow::Result<Arc<str>> {
	let read_api_key = OST_AUTH.read().await;
	match *read_api_key {
		Some(ref api_key) => return Ok(Arc::clone(api_key)),
		None => {
			drop(read_api_key);
			let mut api_key_writable = OST_AUTH.write().await;
			if let Some(ref api_key) = *api_key_writable {
				return Ok(Arc::clone(api_key));
			}
			let login_creds: (String, String) = interact(|| -> anyhow::Result<(String, String)> {
				let username = Input::with_theme(&*THEME)
					.with_prompt("OST Username: ")
					.allow_empty(false)
					.interact_text()?;
				let password = Password::with_theme(&*THEME)
					.with_prompt("Password")
					.allow_empty_password(false)
					.interact()?;
				return Ok((username, password));
			})
			.await?;

			let response: LoginResponse = HTTP_CLIENT
				.post("https://api.opensubtitles.com/api/v1/login")
				.header("User-Agent", "plex-autotagger")
				.header("Api-Key", &*OST_API_KEY)
				.json(&json!({
					"username": login_creds.0,
					"password": login_creds.1,
				}))
				.send()
				.await
				.context("Failed to authenticate with the OST API")?
				.json()
				.await?;

			let api_key = Arc::from(response.token);
			*api_key_writable = Some(Arc::clone(&api_key));
			return Ok(api_key);
		}
	}
}

#[async_trait]
pub trait AuthenticateOST {
	async fn authenticate_ost(self) -> anyhow::Result<reqwest::RequestBuilder>;
}
#[async_trait]
impl AuthenticateOST for reqwest::RequestBuilder {
	async fn authenticate_ost(self) -> anyhow::Result<Self> {
		let user_token = get_ost_auth().await?;
		return Ok(self
			.header("User-Agent", "plex-autotagger")
			.header("Api-Key", &*OST_API_KEY)
			.header("Authorization", String::from("Bearer ") + &*user_token));
	}
}

#[derive(Debug, Deserialize, Clone)]
struct SearchResults {
	// total_pages: u32,
    // total_count: u32,
    // per_page: u32,
    // page: u32,
    data: Vec<SearchResult>,
}

#[derive(Debug, Deserialize, Clone)]
struct SearchResult {
	// id: String,
	// #[serde(rename = "type")]
	// result_type: String,
	attributes: STAttributes,
}

#[derive(Debug, Deserialize, Clone)]
struct STAttributes {
    // subtitle_id: String,
    language: String,
    // download_count: u32,
    // new_download_count: u32,
    // hearing_impaired: bool,
    // votes: u32,
    // ratings: f32,
    // from_trusted: bool,
    // foreign_parts_only: bool,
    // ai_translated: bool,
    // machine_translated: bool,
    // release: String,
	uploader: OSTUploader,
    files: Vec<STFile>,
}

#[derive(Debug, Deserialize, Clone)]
struct OSTUploader {
	// uploader_id: i32,
	name: String,
	rank: String,
}

#[derive(Debug, Deserialize, Clone)]
struct STFile {
	file_id: u32,
	file_name: String,
}

#[derive(Debug, Clone)]
struct SubtitleSummary {
	name: String,
	file_id: u32,
}

pub async fn get_subtitles(episode: &Episode, prompt_user: bool) -> anyhow::Result<String> {
	let response: SearchResults = HTTP_CLIENT
		.get("https://api.opensubtitles.com/api/v1/subtitles")
		.query(&[("tmdb_id", &episode.id.to_string())])
		.authenticate_ost()
		.await.context("Couldn't authenticate with OST")?
		.send()
		.await.context("Error querying subtitles")?
		.json()
		.await.context("Unsupported subtitle query response")?;

	let files: Vec<SubtitleSummary> = response
		.data
		.iter()
		.flat_map(|subtitle| {
			subtitle
				.attributes
				.files
				.iter()
				.map(|file| SubtitleSummary {
					name: format!(
						"lang: {}, name: {}, uploader: {} ({})",
						subtitle.attributes.language,
						file.file_name,
						subtitle.attributes.uploader.name,
						subtitle.attributes.uploader.rank,
					),
					file_id: file.file_id,
				})
		})
		.collect();

	if files.len() == 0 {
		return Err(anyhow!("No subtitles found for title"));
	}

	let subtitle_file = if prompt_user {
		let user_selection_items: Arc<Vec<String>> =
			Arc::new(files.iter().map(|file| file.name.clone()).collect());
		return interact_async(async move {
			loop {
				let user_selection_items = Arc::clone(&user_selection_items);
				let user_selection = task::spawn_blocking(move || {
					Select::with_theme(&*THEME)
						.with_prompt("Select a file")
						.items(&*user_selection_items)
						.default(0)
						.interact()
				})
				.await??;
				let subtitles = download_subtitles(files[user_selection].file_id)
					.await
					.context("An error occurred while downloading subtitles.")?;
				let preview = task::spawn_blocking(move || {
					Confirm::with_theme(&*THEME)
						.with_prompt("Would you like to preview the file?")
						.interact()
				})
				.await??;
				if preview {
					// Check if user wants this file
					let mut less_handle = Command::new("less")
						.stdin(Stdio::piped())
						.stdout(Stdio::inherit())
						.stderr(Stdio::inherit())
						.spawn()?;
					less_handle
						.stdin
						.as_mut()
						.unwrap()
						.write_all(subtitles.as_bytes())
						.await?;
					less_handle.wait().await?;

					let accept_subtitles = task::spawn_blocking(move || {
						Confirm::with_theme(&*THEME)
							.with_prompt("Use these subtitles for comparison?")
							.interact()
					})
					.await??;

					if accept_subtitles {
						break Ok(subtitles);
					}
				} else {
					break Ok(subtitles);
				}
			}
		})
		.await;
	} else {
		download_subtitles(files[0].file_id)
			.await
			.context("An error occurred while downloading subtitles.")?
	};

	return Ok(subtitle_file);
}

#[derive(Debug, Deserialize, Clone)]
struct DownloadPointer {
	link: String,
	// file_name: String,
	// requests: u32,
	// remaining: u32,
	// message: String,
	// reset_time: String,
	// reset_time_utc: String,
}

async fn download_subtitles(file_id: u32) -> anyhow::Result<String> {
	let pointer: DownloadPointer = HTTP_CLIENT
		.post("https://api.opensubtitles.com/api/v1/download")
		.json(&json!({
			"file_id": file_id,
		}))
		.authenticate_ost()
		.await?
		.send()
		.await?
		.json()
		.await?;
	return Ok(HTTP_CLIENT.get(pointer.link).send().await?.text().await?);
}
