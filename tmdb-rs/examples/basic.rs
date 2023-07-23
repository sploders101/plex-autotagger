use tmdb_async::Client;

#[tokio::main]
async fn main() {
	let client = Client::new(env!("TMDB_API_KEY").to_string());
	let search_result = client.movie_search("Interstellar", Some(2014)).await.unwrap();
	let movie = client.movie_by_id(search_result.results()[0].id(), false, false).await.unwrap();
	println!("{:#?}", movie);
}
