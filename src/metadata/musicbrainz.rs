use super::{AlbumDetails, DiscDetails, TrackDetails};
use discid::DiscId;
use musicbrainz_rs::{
    Fetch,
    entity::{
        discid::Discid as MBDiscid,
        release::Release,
    },
};

pub struct MusicBrainzClient;

impl MusicBrainzClient {
    fn mcn_matches_barcode(mcn: Option<&str>, barcode: Option<&str>) -> bool {
        match (mcn, barcode) {
            (Some(mcn), Some(barcode)) => {
                let barcode_len = barcode.len();
                match barcode_len {
                    12 => mcn.starts_with('0') && &mcn[1..] == barcode,
                    13 => mcn == barcode,
                    _ => false,
                }
            }
            _ => false,
        }
    }

    pub fn query_disc(disc_details: &DiscDetails) -> Result<Vec<String>, Box<dyn std::error::Error>> {
        println!("Querying MusicBrainz for disc ID: {}", disc_details.id);
        
        let result = MBDiscid::fetch()
            .id(&disc_details.id)
            .execute();

        match result {
            Ok(disc_result) => {
                let mut release_ids = Vec::new();
                
                if let Some(releases) = disc_result.releases {
                    println!("Found {} releases", releases.len());
                    
                    for release in releases {
                        if let (Some(mcn), Some(barcode)) = (disc_details.mcn.as_deref(), release.barcode.as_deref()) {
                            if Self::mcn_matches_barcode(Some(mcn), Some(barcode)) {
                                println!("MCN matches barcode, using single release: {}", release.id);
                                return Ok(vec![release.id]);
                            }
                        }
                        release_ids.push(release.id);
                    }
                }
                
                Ok(release_ids)
            }
            Err(e) => {
                println!("No MusicBrainz metadata for discid {}: {}", disc_details.id, e);
                Ok(Vec::new())
            }
        }
    }

    fn get_artist_info_from_release(release: &Release) -> (String, Option<String>, Option<String>) {
        if let Some(ref artist_credit) = release.artist_credit {
            if !artist_credit.is_empty() {
                let mut artist_name = String::new();
                let mut first = true;
                
                for credit in artist_credit {
                    if !first {
                        if let Some(ref joinphrase) = credit.joinphrase {
                            if !joinphrase.is_empty() {
                                artist_name.push_str(joinphrase);
                            } else {
                                artist_name.push_str(", ");
                            }
                        } else {
                            artist_name.push_str(", ");
                        }
                    }
                    artist_name.push_str(&credit.name);
                    first = false;
                }
                
                if artist_credit.len() == 1 {
                    let artist = &artist_credit[0];
                    (artist_name, Some(artist.artist.sort_name.clone()), Some(artist.artist.id.clone()))
                } else {
                    println!("Multiple artists");
                    (artist_name, None, None)
                }
            } else {
                ("Unknown Artist".to_string(), None, None)
            }
        } else {
            ("Unknown Artist".to_string(), None, None)
        }
    }

    fn fill_tracks_from_release(release: &Release, album: &mut AlbumDetails) {
        if let Some(ref media) = release.media {
            for medium in media {
                if let Some(ref tracks) = medium.tracks {
                    album.tracks.clear();
                    let mut track_offset = 0;
                    let mut skip_data_tracks = true;

                    for track in tracks {
                        if skip_data_tracks && track.title == "[data track]" {
                            continue;
                        } else {
                            skip_data_tracks = false;
                            if album.tracks.is_empty() {
                                track_offset = track.position.saturating_sub(1);
                            }
                        }

                        let (track_artist, track_artist_sortname, track_artist_id) = 
                            if let Some(ref artist_credit) = track.artist_credit {
                                if !artist_credit.is_empty() {
                                    let mut artist_name = String::new();
                                    let mut first = true;
                                    
                                    for credit in artist_credit {
                                        if !first {
                                            artist_name.push_str(", ");
                                        }
                                        artist_name.push_str(&credit.name);
                                        first = false;
                                    }
                                    
                                    if artist_credit.len() == 1 {
                                        let artist = &artist_credit[0];
                                        (artist_name, Some(artist.artist.sort_name.clone()), Some(artist.artist.id.clone()))
                                    } else {
                                        (artist_name, None, None)
                                    }
                                } else {
                                    (
                                        album.artist.clone().unwrap_or_else(|| "Unknown Artist".to_string()),
                                        album.artist_sortname.clone(),
                                        album.artist_id.clone()
                                    )
                                }
                            } else {
                                (
                                    album.artist.clone().unwrap_or_else(|| "Unknown Artist".to_string()),
                                    album.artist_sortname.clone(),
                                    album.artist_id.clone()
                                )
                            };

                        let track_details = TrackDetails {
                            number: track.position - track_offset,
                            title: track.title.clone(),
                            duration: track.length,
                            artist: Some(track_artist),
                            artist_sortname: track_artist_sortname,
                            artist_id: track_artist_id,
                            track_id: track.recording.as_ref().map(|r| r.id.clone()),
                            composer: None,
                            composer_sortname: None,
                        };

                        album.tracks.push(track_details);
                    }
                }
                break;
            }
        }
    }

    fn make_album_from_release(release: &Release) -> Option<AlbumDetails> {
        let (artist, artist_sortname, artist_id) = Self::get_artist_info_from_release(release);

        let release_date_str = release.date.as_ref().map(|date| {
            date.0.clone()
        });

        let mut album = AlbumDetails {
            album_id: release.id.clone(),
            title: release.title.clone(),
            artist: Some(artist),
            artist_sortname,
            artist_id,
            release_date: release_date_str,
            country: release.country.clone(),
            disc_number: None,
            disc_count: release.media.as_ref().map(|m| m.len() as u32),
            barcode: release.barcode.clone(),
            tracks: Vec::new(),
            composer: None,
            composer_sortname: None,
        };

        Self::fill_tracks_from_release(release, &mut album);

        Some(album)
    }

    pub fn list_albums(disc: &DiscId) -> Result<Vec<AlbumDetails>, Box<dyn std::error::Error>> {
        let mut disc_details = DiscDetails::from_disc(disc);
        let mut albums = Vec::new();

        disc_details.release_ids = Self::query_disc(&disc_details)?;

        if disc_details.release_ids.is_empty() {
            println!("No releases found for this disc");
            return Ok(albums);
        }

        for release_id in &disc_details.release_ids {
            println!("Fetching release details for: {}", release_id);
            
            match Release::fetch()
                .id(release_id)
                .with_artists()
                .with_recordings()
                .with_media()
                .execute() 
            {
                Ok(release) => {
                    if let Some(album) = Self::make_album_from_release(&release) {
                        albums.push(album);
                    }
                }
                Err(e) => {
                    println!("Failed to fetch release {}: {}", release_id, e);
                }
            }
        }

        Ok(albums)
    }
}
