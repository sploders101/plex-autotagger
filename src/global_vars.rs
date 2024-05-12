use std::sync::Arc;

use lazy_static::lazy_static;
use tokio::sync::RwLock;

lazy_static! {
	pub static ref TMDB_API_KEY: String = std::env::var("TMDB_API_KEY")
		.expect("API key not found. Please specify it with TMDB_API_KEY environment variable");
	pub static ref OST_API_KEY: RwLock<Option<Arc<str>>> = RwLock::new(None);
	pub static ref HTTP_CLIENT: reqwest::Client = reqwest::Client::new();
}
