use discid::{DiscId, Features};
use musicbrainz_rs::{
    Fetch,
    entity::{
        discid::Discid as MBDiscid,
        release::Release,
    },
};

#[derive(Debug, Clone)]
struct DiscDetails {
    id: String,
    mcn: Option<String>,
    url: String,
    release_ids: Vec<String>,
}

#[derive(Debug, Clone)]
struct ArtistDetails {
    id: String,
    name: String,
    sortname: Option<String>,
    disambiguation: Option<String>,
    gender: Option<String>,
    country: Option<String>,
}

#[derive(Debug, Clone)]
struct TrackDetails {
    number: u32,
    title: String,
    duration: Option<u32>, // en millisecondes
    artist: Option<String>,
    artist_sortname: Option<String>,
    artist_id: Option<String>,
    track_id: Option<String>,
    composer: Option<String>,
    composer_sortname: Option<String>,
}

#[derive(Debug, Clone)]
struct AlbumDetails {
    album_id: String,
    title: String,
    artist: Option<String>,
    artist_sortname: Option<String>,
    artist_id: Option<String>,
    release_date: Option<String>,
    country: Option<String>,
    disc_number: Option<u32>,
    disc_count: Option<u32>,
    barcode: Option<String>,
    tracks: Vec<TrackDetails>,
    composer: Option<String>,
    composer_sortname: Option<String>,
}

fn print_disc_info(disc: &DiscId) {
    println!("DiscID         : {}", disc.id());
    println!("FreeDB ID      : {}", disc.freedb_id());
    println!("TOC            : {}", disc.toc_string());
    println!("MCN            : {}", disc.mcn());
    println!("Submission URL : {}", disc.submission_url());
    println!("First track    : {}", disc.first_track_num());
    println!("Last track     : {}", disc.last_track_num());
    println!("Sectors        : {}\n", disc.sectors());

    for track in disc.tracks() {
        println!("Track #{}", track.number);
        println!("    ISRC       : {}", track.isrc);
        println!("    Offset     : {}", track.offset);
        println!("    Sectors    : {}", track.sectors);
    }
}

fn get_disc_details(disc: &DiscId) -> DiscDetails {
    let disc_id = disc.id().to_string();
    let mcn = if disc.mcn().is_empty() { 
        None 
    } else { 
        Some(disc.mcn().to_string()) 
    };
    let url = disc.submission_url().to_string();
    
    println!("Disc id: {}", disc_id);
    println!("Submission URL: {}", url);
    if let Some(ref mcn_val) = mcn {
        println!("Disc MCN: {}", mcn_val);
    }

    DiscDetails {
        id: disc_id,
        mcn,
        url,
        release_ids: Vec::new(),
    }
}

fn mcn_matches_barcode(mcn: Option<&str>, barcode: Option<&str>) -> bool {
    match (mcn, barcode) {
        (Some(mcn), Some(barcode)) => {
            let barcode_len = barcode.len();
            match barcode_len {
                12 => mcn.starts_with('0') && &mcn[1..] == barcode, // UPC barcode
                13 => mcn == barcode, // EAN barcode
                _ => false, // Unknown/invalid barcode
            }
        }
        _ => false,
    }
}

fn query_musicbrainz_disc(disc_details: &DiscDetails) -> Result<Vec<String>, Box<dyn std::error::Error>> {
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
                    // Vérifier si le MCN correspond au code-barres
                    if let (Some(mcn), Some(barcode)) = (disc_details.mcn.as_deref(), release.barcode.as_deref()) {
                        if mcn_matches_barcode(Some(mcn), Some(barcode)) {
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
            
            // Si un seul artiste, retourner ses détails complets
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
                    // Ignorer les pistes de données au début du disque
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
                        composer: None, // À implémenter si nécessaire
                        composer_sortname: None,
                    };

                    album.tracks.push(track_details);
                }
            }
            break; // Traiter seulement le premier medium pour l'instant
        }
    }
}

fn make_album_from_release(release: &Release) -> Option<AlbumDetails> {
    let (artist, artist_sortname, artist_id) = get_artist_info_from_release(release);

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

    // Traiter les pistes
    fill_tracks_from_release(release, &mut album);

    Some(album)
}

fn list_albums(disc: &DiscId) -> Result<Vec<AlbumDetails>, Box<dyn std::error::Error>> {
    let mut disc_details = get_disc_details(disc);
    let mut albums = Vec::new();

    // Obtenir les IDs des releases depuis MusicBrainz
    disc_details.release_ids = query_musicbrainz_disc(&disc_details)?;

    if disc_details.release_ids.is_empty() {
        println!("No releases found for this disc");
        return Ok(albums);
    }

    // Pour chaque release, obtenir les détails complets
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
                if let Some(album) = make_album_from_release(&release) {
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

fn main() {
    gstreamer::init().unwrap();
    let version = gstreamer::version_string();
    println!("{}", version);

    let disc = DiscId::read_features(None, Features::all()).expect("Reading disc failed");

    print_disc_info(&disc);

    // Nouvelle logique MusicBrainz simplifiée
    println!("\n=== MusicBrainz Metadata ===");
    match list_albums(&disc) {
        Ok(albums) => {
            if albums.is_empty() {
                println!("No album metadata found");
            } else {
                for (i, album) in albums.iter().enumerate() {
                    println!("\n--- Album {} ---", i + 1);
                    println!("Title: {}", album.title);
                    if let Some(ref artist) = album.artist {
                        println!("Artist: {}", artist);
                    }
                    if let Some(ref date) = album.release_date {
                        println!("Release Date: {}", date);
                    }
                    if let Some(ref country) = album.country {
                        println!("Country: {}", country);
                    }
                    if let Some(ref barcode) = album.barcode {
                        println!("Barcode: {}", barcode);
                    }
                    
                    println!("Tracks ({}):", album.tracks.len());
                    for track in &album.tracks {
                        println!("  {}: {}", track.number, track.title);
                        if let Some(duration) = track.duration {
                            let seconds = duration / 1000;
                            println!("     Duration: {}:{:02}", seconds / 60, seconds % 60);
                        }
                    }
                }
            }
        }
        Err(e) => {
            println!("Error fetching album metadata: {}", e);
        }
    }



    // let pipeline = Pipeline::new();
    // let source = ElementFactory::make_with_name("filesrc", Some("src")).unwrap();
    // source.set_property_from_str("location", "FallingSky.mp3");
    // let decodebin = ElementFactory::make_with_name("decodebin", Some("decoder")).unwrap();
    // let audiorate = ElementFactory::make("audiorate").build().unwrap();
    // let audioconvert = ElementFactory::make("audioconvert").build().unwrap();
    // let audioresample = ElementFactory::make("audioresample").build().unwrap();

    // // Choix de l'encodeur avec un énum
    // let encoder = OpusEncoder::new();

    // let sink = ElementFactory::make("filesink").build().unwrap();
    // sink.set_property_from_str("location", "FallingSky.opus");

    // pipeline
    //     .add_many(&[
    //         &source,
    //         &decodebin,
    //         &audiorate,
    //         &audioconvert,
    //         &audioresample,
    //         encoder.get_encoder(),
    //         encoder.get_muxer(),
    //         &sink,
    //     ])
    //     .unwrap();

    // source.link(&decodebin).unwrap();
    // audiorate.link(&audioconvert).unwrap();
    // audioconvert.link(&audioresample).unwrap();
    // audioresample.link(encoder.get_encoder()).unwrap();
    // encoder.get_encoder().link(encoder.get_muxer()).unwrap();
    // encoder.get_muxer().link(&sink).unwrap();

    // let audiorate_clone = audiorate.clone();
    // decodebin.connect_pad_added(move |_dbin, src_pad| {
    //     println!("decodebin pad-added: {}", src_pad.name());
    //     if let Some(caps) = src_pad.current_caps() {
    //         if let Some(structure) = caps.structure(0) {
    //             let media_type = structure.name();
    //             println!("Pad caps: {}", media_type);
    //             if !media_type.starts_with("audio/") && media_type != "audio/x-raw" {
    //                 println!("Ignoring non-audio pad: {}", media_type);
    //                 return;
    //             }
    //         }
    //     }
    //     let sink_pad = audiorate_clone
    //         .static_pad("sink")
    //         .expect("audiorate has no sink pad");
    //     match src_pad.link(&sink_pad) {
    //         Ok(PadLinkSuccess) => {
    //             println!("Linked decodebin -> audiorate");
    //         }
    //         Err(err) => {
    //             eprintln!("Failed to link decodebin pad to audiorate: {:?}", err);
    //         }
    //     }
    // });

    // let bus = pipeline
    //     .bus()
    //     .expect("Pipeline without bus — this should not happen");
    // let main_loop = MainLoop::new(None, false);
    // let ml_clone = main_loop.clone();
    // let _bus_watch = bus
    //     .add_watch(move |_bus, msg| {
    //         match msg.view() {
    //             MessageView::Eos(_) => {
    //                 println!("EOS received — stopping main loop");
    //                 ml_clone.quit();
    //             }
    //             MessageView::Error(err) => {
    //                 let src = err
    //                     .src()
    //                     .map(|s| s.path_string())
    //                     .unwrap_or_else(|| glib::GString::from("unknown"));
    //                 eprintln!("Error from {}: {} ({:?})", src, err.error(), err.debug());
    //                 ml_clone.quit();
    //             }
    //             MessageView::StateChanged(_s) => {
    //                 // optional logging
    //             }
    //             _ => {}
    //         }
    //         ControlFlow::Continue
    //     })
    //     .expect("Failed to add bus watch");

    // pipeline
    //     .set_state(State::Playing)
    //     .expect("Unable to set the pipeline to `Playing` state");
    // println!("Running main loop...");
    // main_loop.run();
    // pipeline
    //     .set_state(State::Null)
    //     .expect("Failed to set pipeline to Null state");
    // println!("Done.");
}
