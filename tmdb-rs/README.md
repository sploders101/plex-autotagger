# The Movie Database
![The Movie Database](https://www.themoviedb.org/assets/2/v4/logos/408x161-powered-by-rectangle-green-bb4301c10ddc749b4e79463811a68afebeae66ef43d17bcfd8ff0e60ded7ce99.png)

This is an `async` wrapper around the [TMDb API](https://developers.themoviedb.org/3).

This copy adds extra functionality needed for this tool to function. The original version is hosted on
a personal Gitlab instance at https://gitlab.cronce.io/foss/tmdb-rs. Credits to Mike Cronce for making
this awesome library.

## Usage
```rust
use tmdb_async::Client;

#[tokio::main]
async fn main() {
	let tmdb = Client::new(env!("TMDB_API_KEY").to_string());
	let search_result = client.movie_search("Interstellar", Some(2014)).await.unwrap();
	let movie = client.movie_by_id(search_result.results[0].id, false, false).await.unwrap();
	println!("{:#?}", movie);
}
```

## Actions
Currently there are 3 actions available:
* Fetching by ID
* Searching by name and (optionally) year of release
* Finding by external ID (IMDb ID, TVDB ID)

Additionally, two media types are currently supported:
* Movies
* TV series

### Fetching
If you know its ID, you can fetch a movie using that.

```rust
let movie = tmdb.movie_by_id(157336).await.unwrap();
```

You can request some more data with the [append to response](https://developers.themoviedb.org/3/getting-started/append-to-response) feature.

```rust
let movie = tmdb.movie_by_id(2277, true, true).await.unwrap();
```

### Searching
You can search for movies and series by `title` and `year`.

```rust
let page = tmdb.movie_search("Bicentennial Man", Some(1999)).await.unwrap();
let movies = page.results;
```

If you require additional details that aren't returned by the search, you can search then fetch:

```rust
let page = tmdb.movie_search("Bicentennial Man", Some(1999)).await.unwrap();
let movie = tmdb.movie_by_id(page.results[0].id, true, true).await.unwrap();
```

### Finding
[Finding](https://developers.themoviedb.org/3/find/find-by-id) a movie with an external ID is currently supported with IMDB IDs and, for TV series, TVDB IDs.

```rust
let movie = tmdb.movie_by_imdb_id(816692).await.unwrap();
```

## Acknowledgements
* This library is forked from [tmdb-rs](https://gitlab.com/Cir0X/tmdb-rs)
* [The Movie Database (TMDb)](https://www.themoviedb.org/)

