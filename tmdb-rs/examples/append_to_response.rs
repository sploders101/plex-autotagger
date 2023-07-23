use tmdb_async::Client;

#[tokio::main]
async fn main() {
	let client = Client::new(env!("TMDB_API_KEY").to_string());
	let movie = client.movie_by_id(2277, true, true).await.unwrap();
	println!("{:#?}", movie);
}
