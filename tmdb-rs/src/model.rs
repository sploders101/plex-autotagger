use compact_str::CompactString;
use gset::Getset;
use isocountry::CountryCode;
use isolanguage_1::LanguageCode;
use serde::{Deserialize, Deserializer};
use serde_with::serde_as;
use time::Date;

time::serde::format_description!(date, Date, "[year]-[month]-[day]");

mod optional_date {
	use serde::{de::Visitor, Deserializer, Serializer};
	use time::{format_description, Date};
	use lazy_static::lazy_static;

	lazy_static! {
		static ref DATE_FORMAT: Vec<format_description::FormatItem<'static>> = format_description::parse("[year]-[month]-[day]").unwrap();
	}

	struct OptionalDateVisitor;
	impl<'a> Visitor<'a> for OptionalDateVisitor {
		type Value = Option<Date>;
		fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
			write!(formatter, "an empty string or a date-string of format '[year]-[month]-[day]'")
		}
		fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
		where
			E: serde::de::Error,
		{
			if v == "" {
				return Ok(None);
			}
			return Ok(Some(Date::parse(v, &DATE_FORMAT).map_err(|err| E::custom(err))?));
		}
	}

	pub fn serialize<S>(obj: &Option<Date>, serializer: S) -> Result<S::Ok, S::Error>
	where
		S: Serializer,
	{
		match obj {
			Some(date) => serializer.serialize_str(
				&date
					.format(&DATE_FORMAT)
					.unwrap(),
			),
			None => serializer.serialize_str(""),
		}
	}
	pub fn deserialize<'de, D>(deserializer: D) -> Result<Option<Date>, D::Error>
	where
		D: Deserializer<'de>,
	{
		deserializer.deserialize_any(OptionalDateVisitor)
	}
}

#[inline]
fn country_code<'de, D: Deserializer<'de>>(de: D) -> Result<Option<CountryCode>, D::Error> {
	use serde::de::{Error, Unexpected};
	match Option::<&str>::deserialize(de)? {
		None => Ok(None),
		Some("") => Ok(None),
		Some(s) if s.len() == 2 => match CountryCode::for_alpha2_caseless(s) {
			Ok(v) => Ok(Some(v)),
			Err(_) => Err(D::Error::invalid_value(
				Unexpected::Str(s),
				&"any 2-letter ISO 3166-1 country code",
			)),
		},
		Some(s) => Err(D::Error::invalid_value(
			Unexpected::Str(s),
			&"any 2-letter ISO 3166-1 country code",
		)),
	}
}

mod imdb_id {
	use serde::de::{Error, Unexpected};
	use serde::{Deserialize, Deserializer};

	#[inline]
	fn from_str<'de, D: Deserializer<'de>>(s: &str) -> Result<u32, D::Error> {
		if (s.len() < 3 || &s[..2] != "tt") {
			return Err(D::Error::invalid_value(
				Unexpected::Str(s),
				&"a signed integer prefixed by \"tt\"",
			));
		}
		match s[2..].parse() {
			Ok(v) => Ok(v),
			Err(_) => Err(D::Error::invalid_value(
				Unexpected::Str(s),
				&"a signed integer prefixed by \"tt\"",
			)),
		}
	}

	#[inline]
	pub(crate) fn deserialize<'de, D: Deserializer<'de>>(de: D) -> Result<u32, D::Error> {
		let s = <&str>::deserialize(de)?;
		from_str::<D>(s)
	}

	pub(super) mod option {
		use super::*;
		#[inline]
		pub(crate) fn deserialize<'de, D: Deserializer<'de>>(
			de: D,
		) -> Result<Option<u32>, D::Error> {
			match Option::<&str>::deserialize(de)? {
				Some(s) => Ok(Some(from_str::<D>(s)?)),
				None => Ok(None),
			}
		}
	}
}

#[derive(Debug, Clone, Eq, PartialEq, Deserialize, Getset)]
pub struct Genre {
	#[getset(get_copy, vis = "pub")]
	id: u16,
	#[getset(deref_get, vis = "pub")]
	name: CompactString,
}

#[derive(Debug, Clone, Eq, PartialEq, Deserialize, Getset)]
pub struct Results<T> {
	#[getset(deref_get, vis = "pub")]
	results: Vec<T>,
}

#[derive(Debug, Clone, Eq, PartialEq, Deserialize, Getset)]
pub struct Video {
	#[getset(deref_get, vis = "pub")]
	id: CompactString,
	#[getset(get_copy, vis = "pub")]
	iso_639_1: LanguageCode,
	#[getset(deref_get, vis = "pub")]
	key: CompactString,
	#[getset(deref_get, vis = "pub")]
	name: CompactString,
	#[getset(deref_get, vis = "pub")]
	site: CompactString,
	#[getset(get_copy, vis = "pub")]
	size: u16,
	#[getset(deref_get, vis = "pub")]
	#[serde(rename = "type")]
	video_type: CompactString,
}

#[serde_as]
#[derive(Debug, Clone, Eq, PartialEq, Deserialize, Getset)]
pub struct Cast {
	#[getset(get_copy, vis = "pub")]
	id: u32,
	#[getset(get_copy, vis = "pub")]
	cast_id: u32,
	#[serde_as(as = "serde_with::hex::Hex")]
	#[getset(get_copy, vis = "pub")]
	credit_id: [u8; 12],
	#[getset(deref_get, vis = "pub")]
	character: CompactString,
	#[getset(get_copy, vis = "pub")]
	gender: Option<u8>,
	#[getset(deref_get, vis = "pub")]
	name: CompactString,
	#[getset(as_deref_get, vis = "pub", type = "Option<&str>")]
	profile_path: Option<String>,
	#[getset(get_copy, vis = "pub")]
	order: u8,
}

#[serde_as]
#[derive(Debug, Clone, Eq, PartialEq, Deserialize, Getset)]
pub struct TVCast {
	#[getset(get_copy, vis = "pub")]
	id: u32,
	#[serde_as(as = "serde_with::hex::Hex")]
	#[getset(get_copy, vis = "pub")]
	credit_id: [u8; 12],
	#[getset(deref_get, vis = "pub")]
	character: CompactString,
	#[getset(get_copy, vis = "pub")]
	gender: Option<u8>,
	#[getset(deref_get, vis = "pub")]
	name: CompactString,
	#[getset(as_deref_get, vis = "pub", type = "Option<&str>")]
	profile_path: Option<String>,
	#[getset(get_copy, vis = "pub")]
	order: u32,
}

#[serde_as]
#[derive(Debug, Clone, Eq, PartialEq, Deserialize, Getset)]
pub struct TVCreator {
	#[getset(get_copy, vis = "pub")]
	id: u32,
	#[serde_as(as = "serde_with::hex::Hex")]
	#[getset(get_copy, vis = "pub")]
	credit_id: [u8; 12],
	#[getset(deref_get, vis = "pub")]
	name: CompactString,
	#[getset(get_copy, vis = "pub")]
	gender: Option<u8>,
	#[getset(as_deref_get, vis = "pub", type = "Option<&str>")]
	profile_path: Option<String>,
}

#[serde_as]
#[derive(Debug, Clone, Eq, PartialEq, Deserialize, Getset)]
pub struct Crew {
	#[serde_as(as = "serde_with::hex::Hex")]
	#[getset(get_copy, vis = "pub")]
	credit_id: [u8; 12],
	#[getset(deref_get, vis = "pub")]
	department: CompactString,
	#[getset(get_copy, vis = "pub")]
	gender: Option<u8>,
	#[getset(get_copy, vis = "pub")]
	id: u32,
	#[getset(deref_get, vis = "pub")]
	job: CompactString,
	#[getset(deref_get, vis = "pub")]
	name: CompactString,
	#[getset(as_deref_get, vis = "pub", type = "Option<&str>")]
	profile_path: Option<String>,
}

#[derive(Debug, Clone, Eq, PartialEq, Deserialize, Getset)]
pub struct Credits {
	#[getset(deref_get, vis = "pub")]
	cast: Vec<Cast>,
	#[getset(deref_get, vis = "pub")]
	crew: Vec<Crew>,
}

#[derive(Debug, Clone, Eq, PartialEq, Deserialize, Getset)]
pub struct TVCredits {
	#[getset(deref_get, vis = "pub")]
	cast: Vec<TVCast>,
	#[getset(deref_get, vis = "pub")]
	crew: Vec<Crew>,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Getset)]
pub struct LastEpisode {
	#[serde(with = "optional_date")]
	#[getset(get_copy, vis = "pub")]
	air_date: Option<Date>,
	#[getset(get_copy, vis = "pub")]
	episode_number: u32,
	#[getset(get_copy, vis = "pub")]
	id: u32,
	#[getset(deref_get, vis = "pub")]
	name: CompactString,
	#[getset(deref_get, vis = "pub")]
	overview: String,
	#[getset(as_deref_get, vis = "pub", type = "Option<&str>")]
	production_code: Option<String>,
	#[getset(get_copy, vis = "pub")]
	season_number: u32,
	#[getset(as_deref_get, vis = "pub", type = "Option<&str>")]
	still_path: Option<String>,
	#[getset(get_copy, vis = "pub")]
	vote_average: f64,
	#[getset(get_copy, vis = "pub")]
	vote_count: u64,
}

#[derive(Debug, Clone, Eq, PartialEq, Deserialize, Getset)]
pub struct ProductionCompany {
	#[getset(get_copy, vis = "pub")]
	id: u32,
	#[getset(as_deref_get, vis = "pub", type = "Option<&str>")]
	logo_path: Option<String>,
	#[getset(deref_get, vis = "pub")]
	name: CompactString,
	#[serde(deserialize_with = "country_code")]
	#[getset(get_copy, vis = "pub")]
	origin_country: Option<CountryCode>,
}

#[derive(Debug, Clone, Eq, PartialEq, Deserialize, Getset)]
pub struct Network {
	#[getset(get_copy, vis = "pub")]
	id: u32,
	#[getset(as_deref_get, vis = "pub", type = "Option<&str>")]
	logo_path: Option<String>,
	#[getset(deref_get, vis = "pub")]
	name: CompactString,
	#[serde(deserialize_with = "country_code")]
	#[getset(get_copy, vis = "pub")]
	origin_country: Option<CountryCode>,
}

#[derive(Debug, Clone, Eq, PartialEq, Deserialize, Getset)]
pub struct Season {
	#[serde(with = "date::option")]
	#[getset(get_copy, vis = "pub")]
	air_date: Option<Date>,
	#[getset(get_copy, vis = "pub")]
	episode_count: u32,
	#[getset(get_copy, vis = "pub")]
	id: u32,
	#[getset(deref_get, vis = "pub")]
	name: CompactString,
	#[getset(deref_get, vis = "pub")]
	overview: String,
	#[getset(as_deref_get, vis = "pub", type = "Option<&str>")]
	poster_path: Option<String>,
	#[getset(get_copy, vis = "pub")]
	season_number: u32,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Getset)]
pub struct Movie {
	#[getset(get_copy, vis = "pub")]
	id: u32,
	#[serde(with = "imdb_id")]
	#[getset(get_copy, vis = "pub")]
	imdb_id: u32,
	#[getset(deref_get, vis = "pub")]
	title: CompactString,
	#[getset(deref_get, vis = "pub")]
	tagline: String,
	#[getset(deref_get, vis = "pub")]
	original_title: CompactString,
	#[getset(get_copy, vis = "pub")]
	original_language: LanguageCode,
	#[getset(as_deref_get, vis = "pub", type = "Option<&str>")]
	overview: Option<String>,
	#[serde(with = "optional_date")]
	#[getset(get_copy, vis = "pub")]
	release_date: Option<Date>,
	#[getset(get_copy, vis = "pub")]
	runtime: u32,
	#[getset(as_deref_get, vis = "pub", type = "Option<&str>")]
	homepage: Option<String>,
	#[getset(deref_get, vis = "pub")]
	genres: Vec<Genre>,
	#[getset(as_deref_get, vis = "pub", type = "Option<&str>")]
	poster_path: Option<String>,
	#[getset(as_deref_get, vis = "pub", type = "Option<&str>")]
	backdrop_path: Option<String>,
	#[getset(get_copy, vis = "pub")]
	popularity: f64,
	#[getset(get_copy, vis = "pub")]
	budget: u64,
	#[getset(get_copy, vis = "pub")]
	adult: bool,
	#[getset(as_ref_get, vis = "pub", type = "Option<&Results<Video>>")]
	videos: Option<Results<Video>>,
	#[getset(as_ref_get, vis = "pub", type = "Option<&Credits>")]
	credits: Option<Credits>,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Getset)]
pub struct TV {
	#[getset(get_copy, vis = "pub")]
	id: u32,
	backdrop_path: Option<String>,
	#[getset(deref_get, vis = "pub")]
	created_by: Vec<TVCreator>,
	#[getset(deref_get, vis = "pub")]
	episode_run_time: Vec<u16>,
	#[serde(with = "optional_date")]
	#[getset(get_copy, vis = "pub")]
	first_air_date: Option<Date>,
	#[getset(deref_get, vis = "pub")]
	genres: Vec<Genre>,
	#[getset(as_deref_get, vis = "pub", type = "Option<&str>")]
	homepage: Option<String>,
	#[getset(get_copy, vis = "pub")]
	in_production: bool,
	#[getset(deref_get, vis = "pub")]
	languages: Vec<LanguageCode>,
	#[serde(with = "optional_date")]
	#[getset(get_copy, vis = "pub")]
	last_air_date: Option<Date>,
	#[getset(as_ref_get, vis = "pub", type = "Option<&LastEpisode>")]
	last_episode_to_air: Option<LastEpisode>,
	#[getset(deref_get, vis = "pub")]
	name: CompactString,
	#[getset(deref_get, vis = "pub")]
	networks: Vec<Network>,
	#[getset(get_copy, vis = "pub")]
	number_of_episodes: u32,
	#[getset(get_copy, vis = "pub")]
	number_of_seasons: u32,
	#[getset(deref_get, vis = "pub")]
	origin_country: Vec<CountryCode>,
	#[getset(get_copy, vis = "pub")]
	original_language: LanguageCode,
	#[getset(deref_get, vis = "pub")]
	original_name: CompactString,
	#[getset(deref_get, vis = "pub")]
	overview: String,
	#[getset(get_copy, vis = "pub")]
	popularity: f64,
	#[getset(as_deref_get, vis = "pub", type = "Option<&str>")]
	poster_path: Option<CompactString>,
	#[getset(deref_get, vis = "pub")]
	production_companies: Vec<ProductionCompany>,
	#[getset(deref_get, vis = "pub")]
	seasons: Vec<Season>,
	#[getset(deref_get, vis = "pub")]
	status: CompactString,
	#[getset(deref_get, vis = "pub")]
	r#type: CompactString,
	#[getset(get_copy, vis = "pub")]
	vote_average: f64,
	#[getset(get_copy, vis = "pub")]
	vote_count: u64,
	#[getset(as_ref_get, vis = "pub", type = "Option<&Results<Video>>")]
	videos: Option<Results<Video>>,
	#[getset(as_ref_get, vis = "pub", type = "Option<&TVCredits>")]
	credits: Option<TVCredits>,
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct TVSeason {
	#[serde(with = "optional_date")]
	pub air_date: Option<Date>,
	pub episodes: Vec<Episode>,
	pub name: String,
	pub overview: String,
	pub id: u32,
	pub poster_path: String,
	pub season_number: u32,
	pub vote_average: f32,
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct Episode {
	#[serde(with = "optional_date")]
	pub air_date: Option<Date>,
	pub episode_number: u32,
	pub id: u32,
	pub name: String,
	pub overview: Option<String>,
	pub production_code: Option<String>,
	pub runtime: Option<u32>,
	pub season_number: u32,
	pub show_id: u32,
	pub still_path: Option<String>,
	pub vote_average: Option<f32>,
	pub vote_count: Option<u32>,
	pub crew: Option<Vec<Crew>>,
	// pub guest_stars: Vec<Cast>
}

#[derive(Debug, Clone, PartialEq, Deserialize, Getset)]
pub struct SearchMovie {
	#[getset(get_copy, vis = "pub")]
	id: u32,
	#[getset(deref_get, vis = "pub")]
	title: CompactString,
	#[getset(deref_get, vis = "pub")]
	original_title: CompactString,
	#[getset(get_copy, vis = "pub")]
	original_language: LanguageCode,
	#[getset(as_deref_get, vis = "pub", type = "Option<&str>")]
	overview: Option<String>,
	#[serde(with = "optional_date")]
	#[getset(get_copy, vis = "pub")]
	release_date: Option<Date>,
	#[getset(deref_get, vis = "pub")]
	genre_ids: Vec<u16>,
	#[getset(as_deref_get, vis = "pub", type = "Option<&str>")]
	poster_path: Option<String>,
	#[getset(as_deref_get, vis = "pub", type = "Option<&str>")]
	backdrop_path: Option<String>,
	#[getset(get_copy, vis = "pub")]
	popularity: f64,
	#[getset(get_copy, vis = "pub")]
	adult: bool,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Getset)]
pub struct SearchTV {
	#[getset(get_copy, vis = "pub")]
	id: u32,
	#[getset(deref_get, vis = "pub")]
	name: CompactString,
	#[getset(deref_get, vis = "pub")]
	original_name: CompactString,
	#[getset(get_copy, vis = "pub")]
	original_language: LanguageCode,
	#[getset(as_deref_get, vis = "pub", type = "Option<&str>")]
	overview: Option<String>,
	#[serde(with = "optional_date")]
	#[getset(get_copy, vis = "pub")]
	first_air_date: Option<Date>,
	#[getset(deref_get, vis = "pub")]
	genre_ids: Vec<u16>,
	#[getset(as_deref_get, vis = "pub", type = "Option<&str>")]
	poster_path: Option<String>,
	#[getset(as_deref_get, vis = "pub", type = "Option<&str>")]
	backdrop_path: Option<String>,
	#[getset(get_copy, vis = "pub")]
	popularity: f64,
	#[getset(get_copy, vis = "pub")]
	vote_average: f32,
	#[getset(get_copy, vis = "pub")]
	vote_count: u32,
}

#[derive(Debug, Clone, Eq, PartialEq, Deserialize, Getset)]
pub struct FindMovie {
	#[getset(get_copy, vis = "pub")]
	id: u32,
	#[getset(deref_get, vis = "pub")]
	title: CompactString,
	#[getset(deref_get, vis = "pub")]
	original_title: CompactString,
	#[getset(get_copy, vis = "pub")]
	original_language: LanguageCode,
	#[getset(as_deref_get, vis = "pub", type = "Option<&str>")]
	overview: Option<String>,
	#[serde(with = "optional_date")]
	#[getset(get_copy, vis = "pub")]
	release_date: Option<Date>,
	#[getset(deref_get, vis = "pub")]
	genre_ids: Vec<u16>,
	#[getset(as_deref_get, vis = "pub", type = "Option<&str>")]
	poster_path: Option<String>,
	#[getset(as_deref_get, vis = "pub", type = "Option<&str>")]
	backdrop_path: Option<String>,
	#[getset(get_copy, vis = "pub")]
	adult: bool,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Getset)]
pub struct FindTV {
	#[getset(get_copy, vis = "pub")]
	id: u32,
	#[getset(deref_get, vis = "pub")]
	name: CompactString,
	#[getset(deref_get, vis = "pub")]
	original_name: CompactString,
	#[getset(get_copy, vis = "pub")]
	original_language: LanguageCode,
	#[getset(as_deref_get, vis = "pub", type = "Option<&str>")]
	overview: Option<String>,
	#[serde(with = "optional_date")]
	#[getset(get_copy, vis = "pub")]
	first_air_date: Option<Date>,
	#[getset(deref_get, vis = "pub")]
	genre_ids: Vec<u16>,
	#[getset(as_deref_get, vis = "pub", type = "Option<&str>")]
	poster_path: Option<String>,
	#[getset(as_deref_get, vis = "pub", type = "Option<&str>")]
	backdrop_path: Option<String>,
	#[getset(get_copy, vis = "pub")]
	popularity: f64,
	#[getset(get_copy, vis = "pub")]
	vote_average: f32,
	#[getset(get_copy, vis = "pub")]
	vote_count: u32,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Getset)]
pub struct MovieSearchResult {
	#[getset(get_copy, vis = "pub")]
	page: u8,
	#[getset(get_copy, vis = "pub")]
	total_results: u8,
	#[getset(get_copy, vis = "pub")]
	total_pages: u8,
	#[getset(deref_get, vis = "pub")]
	results: Vec<SearchMovie>,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Getset)]
pub struct TVSearchResult {
	#[getset(get_copy, vis = "pub")]
	page: u8,
	#[getset(get_copy, vis = "pub")]
	total_results: u8,
	#[getset(get_copy, vis = "pub")]
	total_pages: u8,
	#[getset(deref_get, vis = "pub")]
	results: Vec<SearchTV>,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Getset)]
pub struct FindResult {
	#[getset(deref_get, vis = "pub")]
	movie_results: Vec<FindMovie>,
	#[getset(deref_get, vis = "pub")]
	tv_results: Vec<FindTV>,
}

#[derive(Debug, Clone, Eq, PartialEq, Deserialize, Getset)]
pub struct TVExternalIds {
	#[getset(get_copy, vis = "pub")]
	id: u32,
	#[serde(with = "imdb_id::option")]
	#[getset(get_copy, vis = "pub")]
	imdb_id: Option<u32>,
	freebase_mid: Option<CompactString>,
	freebase_id: Option<CompactString>,
	#[getset(get_copy, vis = "pub")]
	tvdb_id: Option<u32>,
	#[getset(get_copy, vis = "pub")]
	tvrage_id: Option<u32>,
	facebook_id: Option<CompactString>,
	instagram_id: Option<CompactString>,
	twitter_id: Option<CompactString>,
}

impl TVExternalIds {
	#[inline]
	pub fn freebase_mid(&self) -> Option<&str> {
		self.freebase_mid.as_deref()
	}

	#[inline]
	pub fn freebase_id(&self) -> Option<&str> {
		self.freebase_id.as_deref()
	}

	#[inline]
	pub fn facebook_id(&self) -> Option<&str> {
		self.facebook_id.as_deref()
	}

	#[inline]
	pub fn instagram_id(&self) -> Option<&str> {
		self.instagram_id.as_deref()
	}

	#[inline]
	pub fn twitter_id(&self) -> Option<&str> {
		self.twitter_id.as_deref()
	}
}
