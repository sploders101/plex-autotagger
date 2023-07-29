#![allow(unused_parens)]
#![warn(clippy::future_not_send)]
use cervine::Cow;
use compact_str::format_compact;
use compact_str::CompactString;
use compact_str::ToCompactString;
use itertools::Itertools;
use model::TVSeason;
pub use reqwest::Error;
use serde::de::DeserializeOwned;

mod model;
use model::FindResult;
pub use model::{Episode, Movie, MovieSearchResult, TVExternalIds, TVSearchResult, TV};

#[cfg(test)]
mod integration_tests;

const BASE_URL: &str = "https://api.themoviedb.org/3";

#[derive(Debug, Clone)]
pub struct Client {
	http: reqwest::Client,
	api_key: String,
	language: CompactString
}

#[inline]
fn compact_str_url(k: &str, v: &str) -> CompactString {
	format_compact!("{k}={v}")
}

impl Client {
	pub fn new(api_key: String) -> Self {
		Self::with_language(api_key, "en")
	}

	#[inline]
	pub fn with_language(api_key: String, language: &str) -> Self {
		Self{
			http: reqwest::Client::new(),
			api_key,
			language: language.into()
		}
	}
	
	#[inline]
	async fn get<T: DeserializeOwned>(&self, path: &str, args: &[(&'static str, Cow<'_, CompactString, str>)]) -> Result<T, Error> {
		let url = format!(
			"{}{}?api_key={}&language={}&{}",
			BASE_URL,
			path,
			self.api_key,
			self.language,
			args.iter().map(|(k, v)| compact_str_url(k, v)).join("&")
		);
		let response = self.http.get(&url).send().await?;
		eprintln!("{}, {:?}", &url, response.status());
		response.json().await
	}

	#[inline]
	pub async fn movie_search(&self, title: &str, year: Option<u16>) -> Result<MovieSearchResult, Error> {
		let mut args = Vec::with_capacity(3);
		args.push(("query", Cow::Borrowed(title)));
		if let Some(year) = year {
			args.push(("year", Cow::Owned(year.to_compact_string())));
		}
		args.push(("append_to_response", Cow::Borrowed("images")));
		self.get("/search/movie", &args).await
	}

	#[inline]
	pub async fn movie_by_id(&self, id: u32, include_videos: bool, include_credits: bool) -> Result<Movie, Error> {
		let args = match (include_videos, include_credits) {
			(false, false) => None,
			(true, false) => Some(("append_to_response", Cow::Borrowed("videos"))),
			(false, true) => Some(("append_to_response", Cow::Borrowed("credits"))),
			(true, true) => Some(("append_to_response", Cow::Borrowed("videos,credits")))
		};
		let path = format_compact!("/movie/{}", id);
		self.get(&path, args.as_ref().map(core::slice::from_ref).unwrap_or_default()).await
	}

	#[inline]
	pub async fn movie_by_imdb_id(&self, id: u32) -> Result<Movie, Error> {
		let path = format_compact!("/find/tt{:07}", id);
		let result: FindResult = self.get(&path, &[
			("external_source", Cow::Borrowed("imdb_id")),
			("append_to_response", Cow::Borrowed("images"))
		]).await?;
		self.movie_by_id(result.movie_results()[0].id(), false, false).await
	}

	#[inline]
	pub async fn tv_search(&self, title: &str, year: Option<u16>) -> Result<TVSearchResult, Error> {
		let mut args = Vec::with_capacity(3);
		args.push(("query", Cow::Borrowed(title)));
		if let Some(year) = year {
			args.push(("year", Cow::Owned(year.to_compact_string())));
		}
		args.push(("append_to_response", Cow::Borrowed("images")));
		self.get("/search/tv", &args).await
	}

	#[inline]
	pub async fn tv_by_id(&self, id: u32, include_videos: bool, include_credits: bool) -> Result<TV, Error> {
		let args = match (include_videos, include_credits) {
			(false, false) => None,
			(true, false) => Some(("append_to_response", Cow::Borrowed("videos"))),
			(false, true) => Some(("append_to_response", Cow::Borrowed("credits"))),
			(true, true) => Some(("append_to_response", Cow::Borrowed("videos,credits")))
		};
		let path = format_compact!("/tv/{}", id);
		self.get(&path, args.as_ref().map(core::slice::from_ref).unwrap_or_default()).await
	}

	#[inline]
	pub async fn season(&self, id: u32, season_id: u32) -> Result<TVSeason, Error> {
		let path = format_compact!("/tv/{}/season/{}", id, season_id);
		self.get(&path, &[]).await
	}

	#[inline]
	pub async fn tv_by_imdb_id(&self, id: u32) -> Result<TV, Error> {
		let path = format_compact!("/find/tt{:07}", id);
		let result: FindResult = self.get(&path, &[
			("external_source", Cow::Borrowed("imdb_id")),
			("append_to_response", Cow::Borrowed("images"))
		]).await?;
		self.tv_by_id(result.tv_results()[0].id(), false, false).await
	}

	#[inline]
	pub async fn tv_by_tvdb_id(&self, id: u32) -> Result<TV, Error> {
		let path = format_compact!("/find/{}", id);
		let result: FindResult = self.get(&path, &[
			("external_source", Cow::Borrowed("tvdb_id")),
			("append_to_response", Cow::Borrowed("images"))
		]).await?;
		self.tv_by_id(result.tv_results()[0].id(), false, false).await
	}

	#[inline]
	pub async fn tv_external_ids(&self, id: u32) -> Result<TVExternalIds, Error> {
		let path = format_compact!("/tv/{}/external_ids", id);
		self.get(&path, &[]).await
	}
}

