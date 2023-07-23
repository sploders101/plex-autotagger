use isocountry::CountryCode;
use isolanguage_1::LanguageCode;

use crate::Client;

const API_KEY: &str = env!("TMDB_API_KEY");

fn client() -> Client {
	Client::new(API_KEY.into())
}

#[tokio::test]
async fn fetch_movie() {
	let client = client();

	let movie = client.movie_by_id(157336, false, false).await.unwrap();
	assert_eq!("Interstellar", movie.original_title());
	assert_eq!("2014-11-05", movie.release_date().to_string());
	assert_eq!(LanguageCode::En, movie.original_language());

	let movie = client.movie_by_id(579974, false, false).await.unwrap();
	assert_eq!("రౌద్రం రణం రుధిరం", movie.original_title());
	assert_eq!("2022-03-24", movie.release_date().to_string());
	assert_eq!(LanguageCode::Te, movie.original_language());
}

#[tokio::test]
async fn fetch_movie_languages() {
	let client = Client::with_language(API_KEY.into(), "en".into());
	let movie = client.movie_by_id(2277, false, false).await.unwrap();
	assert_eq!("Bicentennial Man", movie.title());
	assert_eq!(LanguageCode::En, movie.original_language());

	let client = Client::with_language(API_KEY.into(), "de".into());
	let movie = client.movie_by_id(2277, false, false).await.unwrap();
	assert_eq!("Der 200 Jahre Mann", movie.title());

	let client = Client::with_language(API_KEY.into(), "es".into());
	let movie = client.movie_by_id(2277, false, false).await.unwrap();
	assert_eq!("El hombre bicentenario", movie.title());
}

#[tokio::test]
async fn fetch_movie_append_to_response() {
	let client = client();
	let movie = client.movie_by_id(2277, true, true).await.unwrap();
	assert_eq!(true, movie.videos().is_some());
	assert_eq!(true, movie.credits().is_some());
}

#[tokio::test]
async fn search_movie() {
	let client = client();
	let page = client.movie_search("Bicentennial Man", Some(1999)).await.unwrap();

	assert_eq!(1, page.total_results());
	assert_eq!("Bicentennial Man", page.results()[0].title());
}

#[tokio::test]
async fn find_movie_by_imdb_id() {
	let client = client();
	let movie = client.movie_by_imdb_id(816692).await.unwrap();

	assert_eq!("Interstellar", movie.title());
}

#[tokio::test]
async fn fetch_searched_movie() {
	let client = client();
	let page = client.movie_search("Bicentennial Man", Some(1999)).await.unwrap();
	let movie = client.movie_by_id(page.results()[0].id(), false, false).await.unwrap();

	assert_eq!(2277, movie.id());
}

#[tokio::test]
async fn fetch_tv() {
	let client = client();

	let series = client.tv_by_id(2316, false, false).await.unwrap();
	assert_eq!("The Office", series.original_name());
	assert_eq!("2005-03-24", series.first_air_date().to_string());
	assert_eq!("2013-05-16", series.last_air_date().to_string());
	assert_eq!("2013-05-16", series.last_episode_to_air().unwrap().air_date().to_string());
	assert_eq!("2006-07-13", series.seasons()[0].air_date().unwrap().to_string());
	assert_eq!("2005-03-24", series.seasons()[1].air_date().unwrap().to_string());
	assert_eq!("2005-09-20", series.seasons()[2].air_date().unwrap().to_string());
	assert_eq!("2006-09-21", series.seasons()[3].air_date().unwrap().to_string());
	assert_eq!("2007-09-27", series.seasons()[4].air_date().unwrap().to_string());
	assert_eq!("2008-09-25", series.seasons()[5].air_date().unwrap().to_string());
	assert_eq!("2009-09-17", series.seasons()[6].air_date().unwrap().to_string());
	assert_eq!("2010-09-23", series.seasons()[7].air_date().unwrap().to_string());
	assert_eq!("2011-09-22", series.seasons()[8].air_date().unwrap().to_string());
	assert_eq!("2012-09-20", series.seasons()[9].air_date().unwrap().to_string());
	assert_eq!(&[LanguageCode::En], series.languages());
	assert_eq!(LanguageCode::En, series.original_language());
	assert_eq!(Some(CountryCode::USA), series.networks()[0].origin_country());
	assert_eq!(&[CountryCode::USA], series.origin_country());
	assert_eq!("525730af760ee3776a344e89", hex::encode(&series.created_by()[0].credit_id()));
	assert_eq!("525730af760ee3776a344e8f", hex::encode(&series.created_by()[1].credit_id()));
	assert_eq!("525730af760ee3776a344e95", hex::encode(&series.created_by()[2].credit_id()));

	let series = client.tv_by_id(45, false, false).await.unwrap();
	assert_eq!("Top Gear", series.original_name());
	assert_eq!("2002-10-20", series.first_air_date().to_string());
	assert_eq!(Some(CountryCode::GBR), series.networks()[0].origin_country());
	assert_eq!(Some(CountryCode::GBR), series.networks()[1].origin_country());
	assert_eq!(&[CountryCode::GBR], series.origin_country());
	assert_eq!(Some(CountryCode::GBR), series.production_companies()[0].origin_country());
	assert_eq!("5681665fc3a36828f50068ac", hex::encode(&series.created_by()[0].credit_id()));
}

#[tokio::test]
async fn fetch_tv_languages() {
	// TODO:  The Flash might have been a poor choice, since its name appears
	//    to be the same in all languages.  Let's find a show that differs from
	//    English to German to Spanish.
	let client = Client::with_language(API_KEY.into(), "en".into());
	let series = client.tv_by_id(60735, false, false).await.unwrap();
	assert_eq!("The Flash", series.name());

	let client = Client::with_language(API_KEY.into(), "de".into());
	let series = client.tv_by_id(60735, false, false).await.unwrap();
	assert_eq!("The Flash", series.name());

	let client = Client::with_language(API_KEY.into(), "es".into());
	let series = client.tv_by_id(60735, false, false).await.unwrap();
	assert_eq!("The Flash", series.name());
}

#[tokio::test]
async fn fetch_tv_append_to_response() {
	let client = client();
	let series = client.tv_by_id(2316, true, true).await.unwrap();

	assert_eq!(true, series.videos().is_some());
	assert_eq!(true, series.credits().is_some());
}

#[tokio::test]
async fn search_tv() {
	let client = client();
	let page = client.tv_search("The Simpsons", Some(1989)).await.unwrap();

	assert_eq!(2, page.total_results());
	assert_eq!("The Simpsons", page.results()[0].name());
	assert_eq!("Icons Unearthed: The Simpsons", page.results()[1].name());
}

#[tokio::test]
async fn find_tv_by_imdb_id() {
	let client = client();
	let series = client.tv_by_imdb_id(1520211).await.unwrap();

	assert_eq!("The Walking Dead", series.name());
}

#[tokio::test]
async fn find_tv_by_tvdb_id() {
	let client = client();
	let series = client.tv_by_tvdb_id(94571).await.unwrap();

	assert_eq!("Community", series.name());
}

#[tokio::test]
async fn fetch_searched_tv() {
	let client = client();
	let page = client.tv_search("Futurama", None).await.unwrap();
	let series = client.tv_by_id(page.results()[0].id(), false, false).await.unwrap();

	assert_eq!(615, series.id());
}

#[tokio::test]
async fn tv_external_ids() {
	let client = client();
	let ids = client.tv_external_ids(52814).await.unwrap();

	assert_eq!(ids.id(), 52814);
	assert_eq!(ids.imdb_id(), Some(2934286));
	assert_eq!(ids.freebase_mid(), Some("/m/0w1g888".into()));
	assert_eq!(ids.freebase_id(), None);
	assert_eq!(ids.tvdb_id(), Some(366524));
	assert_eq!(ids.tvrage_id(), None);
	assert_eq!(ids.facebook_id(), Some("HaloTheSeries".into()));
	assert_eq!(ids.instagram_id(), Some("halotheseries".into()));
	assert_eq!(ids.twitter_id(), Some("HaloTheSeries".into()));
}

