use anyhow::{Context, anyhow};
use dialoguer::Select;
use matroska::{Track, Tracktype};
use std::path::Path;

use crate::{interact::interact, THEME};

/// Gets the track to be used for comparison with OST, attempting to automatically
/// deduce the best one or by prompting the user.
pub async fn get_comparison_track(file: &Path) -> anyhow::Result<Track> {
    let mut tracks = get_subtitle_tracks(file)?;
	if tracks.len() == 0 {
		return Err(anyhow!("No valid subtitle tracks found"));
	}
    let default_track = get_default_track(&tracks).cloned();
    let selected_track = match default_track {
        Some(track) => track,
        None => {
            let track_option_strings = tracks
                .iter()
                .map(|track| {
                    format!(
                        "Track {}: uid {}, codec {}, language: {:?}",
                        &track.number, &track.uid, &track.codec_id, &track.language
                    )
                })
                .collect::<Vec<_>>();
			let selection_index = interact(move || {
				Select::with_theme(&*THEME)
					.items(&track_option_strings)
					.with_prompt("Select the subtitles track to use for comparison")
					.interact()
			}).await?;

			tracks.swap_remove(selection_index)
        }
    };

	return Ok(selected_track);
}

/// Tries to narrow down which subtitles track is preferrable.
/// If we are able to narrow it down to exactly one, it is returned.
/// Otherwise, this function returns None.
fn get_default_track<'a>(tracks: &'a Vec<Track>) -> Option<&'a Track> {
    return match tracks.len() {
        0 => None,
        1 => Some(&tracks[0]),
        _ => {
            let mut default_tracks = tracks.iter().filter(|track| track.default);
            if let (Some(track), None) = (default_tracks.next(), default_tracks.next()) {
                Some(track)
            } else {
                None
            }
        }
    };
}

/// Gets a list of subtitle tracks from an MKV file
fn get_subtitle_tracks(file: &Path) -> anyhow::Result<Vec<Track>> {
    let vid = matroska::open(file).context("Couldn't open video file")?;
    let tracks: Vec<Track> = vid
        .tracks
        .into_iter()
        .filter(|track| track.tracktype == Tracktype::Subtitle && track.enabled)
        .filter(|track| {
            track
                .language
                .as_ref()
                .map(|lang| match lang {
                    matroska::Language::ISO639(lang) if lang == "eng" => true,
                    matroska::Language::IETF(lang) if lang == "en" => true,
                    matroska::Language::IETF(lang) if lang == "en-US" => true,
                    matroska::Language::IETF(lang) if lang == "en-GB" => true,
                    _ => false,
                })
                .unwrap_or(false)
        })
        .collect();
    return Ok(tracks);
}
