use tmdb_async::Client;

#[tokio::main]
async fn main() {
	let client = Client::new(env!("TMDB_API_KEY").to_string());
	let movie = client.movie_by_imdb_id(816692).await.unwrap();
	println!("Movies: {:#?}", movie);
}
