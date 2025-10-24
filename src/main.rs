use discid::{DiscId, Features};
use musicbrainz_rs::{
    Fetch,
    entity::{
        discid::Discid as MBDiscid,
        release::Release,
    },
};

use glib::{ControlFlow, MainLoop, object::ObjectExt};
use gstreamer::{
    ElementFactory, MessageView, Pipeline, State, prelude::*,
};
use gstreamer_pbutils::{EncodingAudioProfile, EncodingContainerProfile};
use gtk::glib;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum AudioFormat {
    Opus,
    Vorbis,
    Flac,
    Mp3,
    Aac,
    Wavpack,
}

impl AudioFormat {
    fn file_extension(&self) -> &str {
        match self {
            AudioFormat::Opus => "opus",
            AudioFormat::Vorbis => "ogg",
            AudioFormat::Flac => "flac",
            AudioFormat::Mp3 => "mp3",
            AudioFormat::Aac => "m4a",
            AudioFormat::Wavpack => "wv",
        }
    }

    fn name(&self) -> &str {
        match self {
            AudioFormat::Opus => "Opus",
            AudioFormat::Vorbis => "Ogg Vorbis",
            AudioFormat::Flac => "FLAC",
            AudioFormat::Mp3 => "MP3",
            AudioFormat::Aac => "AAC",
            AudioFormat::Wavpack => "WavPack",
        }
    }

    fn is_lossless(&self) -> bool {
        matches!(self, AudioFormat::Flac | AudioFormat::Wavpack)
    }
}

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

fn sanitize_filename(name: &str) -> String {
    name.chars()
        .map(|c| match c {
            '/' | '\\' | ':' | '*' | '?' | '"' | '<' | '>' | '|' => '_',
            c => c,
        })
        .collect()
}

fn create_encoding_profile(format: AudioFormat) -> EncodingContainerProfile {
    match format {
        AudioFormat::Opus => {
            let opus_caps = gstreamer::Caps::builder("audio/x-opus").build();
            let audio_profile = EncodingAudioProfile::builder(&opus_caps).build();
            
            let ogg_caps = gstreamer::Caps::builder("application/ogg").build();
            EncodingContainerProfile::builder(&ogg_caps)
                .add_profile(audio_profile)
                .build()
        }
        AudioFormat::Vorbis => {
            let vorbis_caps = gstreamer::Caps::builder("audio/x-vorbis").build();
            let audio_profile = EncodingAudioProfile::builder(&vorbis_caps).build();
            
            let ogg_caps = gstreamer::Caps::builder("application/ogg").build();
            EncodingContainerProfile::builder(&ogg_caps)
                .add_profile(audio_profile)
                .build()
        }
        AudioFormat::Flac => {
            let flac_caps = gstreamer::Caps::builder("audio/x-flac").build();
            let audio_profile = EncodingAudioProfile::builder(&flac_caps).build();
            
            // FLAC peut être dans un conteneur Ogg ou autonome
            let flac_container_caps = gstreamer::Caps::builder("audio/x-flac").build();
            EncodingContainerProfile::builder(&flac_container_caps)
                .add_profile(audio_profile)
                .build()
        }
        AudioFormat::Mp3 => {
            // Pour MP3, on utilise LAME + ID3v2
            let mp3_caps = gstreamer::Caps::builder("audio/mpeg")
                .field("mpegversion", 1)
                .field("layer", 3)
                .build();
            let audio_profile = EncodingAudioProfile::builder(&mp3_caps).build();
            
            let id3_caps = gstreamer::Caps::builder("application/x-id3").build();
            EncodingContainerProfile::builder(&id3_caps)
                .add_profile(audio_profile)
                .build()
        }
        AudioFormat::Aac => {
            let aac_caps = gstreamer::Caps::builder("audio/mpeg")
                .field("mpegversion", 4)
                .build();
            let audio_profile = EncodingAudioProfile::builder(&aac_caps).build();
            
            let mp4_caps = gstreamer::Caps::builder("video/quicktime")
                .field("variant", "iso")
                .build();
            EncodingContainerProfile::builder(&mp4_caps)
                .add_profile(audio_profile)
                .build()
        }
        AudioFormat::Wavpack => {
            let wavpack_caps = gstreamer::Caps::builder("audio/x-wavpack").build();
            let audio_profile = EncodingAudioProfile::builder(&wavpack_caps).build();
            
            let wv_caps = gstreamer::Caps::builder("audio/x-wavpack").build();
            EncodingContainerProfile::builder(&wv_caps)
                .add_profile(audio_profile)
                .build()
        }
    }
}

fn create_output_filename(track: &TrackDetails, album: &AlbumDetails, format: AudioFormat) -> String {
    let track_num = format!("{:02}", track.number);
    let title = sanitize_filename(&track.title);
    let artist = sanitize_filename(track.artist.as_ref().unwrap_or(&"Unknown".to_string()));
    let album_title = sanitize_filename(&album.title);
    let extension = format.file_extension();
    
    format!("{} - {} - {} - {}.{}", track_num, artist, album_title, title, extension)
}

fn transcode_cd_track(
    _disc: &DiscId,
    track: &TrackDetails, 
    album: &AlbumDetails, 
    output_filename: &str,
    format: AudioFormat,
) -> Result<(), Box<dyn std::error::Error>> {
    println!("Transcodage de la piste {} : {} (format: {})", track.number, track.title, format.name());
    
    let pipeline = Pipeline::new();
    
    // Source CD audio
    let source = ElementFactory::make_with_name("cdparanoiasrc", Some("src")).unwrap();
    source.set_property("track", track.number as u32);
    
    // Pipeline de traitement audio
    let audiorate = ElementFactory::make("audiorate").build().unwrap();
    let audioconvert = ElementFactory::make("audioconvert").build().unwrap();
    let audioresample = ElementFactory::make("audioresample").build().unwrap();
    
    // Créer encodebin avec le profil d'encodage
    let encodebin = ElementFactory::make("encodebin").build().unwrap();
    let profile = create_encoding_profile(format);
    encodebin.set_property("profile", &profile);
    
    // Sink pour le fichier de sortie
    let sink = ElementFactory::make("filesink").build().unwrap();
    sink.set_property_from_str("location", output_filename);

    // Ajouter tous les éléments au pipeline
    pipeline.add_many(&[
        &source,
        &audiorate,
        &audioconvert,
        &audioresample,
        &encodebin,
        &sink,
    ])?;

    // Lier les éléments (encodebin sera lié dynamiquement)
    source.link(&audiorate)?;
    audiorate.link(&audioconvert)?;
    audioconvert.link(&audioresample)?;
    
    // Demander un pad audio à encodebin
    let audio_pad = encodebin.request_pad_simple("audio_%u")
        .ok_or_else(|| format!("Impossible de créer un pad audio pour encodebin. Le format {} n'est peut-être pas supporté ou les plugins nécessaires ne sont pas installés.", format.name()))?;
    let audioresample_src_pad = audioresample.static_pad("src")
        .ok_or("Impossible d'obtenir le pad source d'audioresample")?;
    audioresample_src_pad.link(&audio_pad)?;
    
    // Lier encodebin au sink
    encodebin.link(&sink)?;

    // Appliquer les métadonnées
    apply_metadata_to_pipeline(&pipeline, track, album)?;

    // Créer le bus pour les messages
    let bus = pipeline.bus().expect("Pipeline without bus");
    let main_loop = MainLoop::new(None, false);
    let ml_clone = main_loop.clone();
    
    // Cloner les données nécessaires pour le closure
    let track_number = track.number;
    
    let _bus_watch = bus.add_watch(move |_bus, msg| {
        match msg.view() {
            MessageView::Eos(_) => {
                println!("Transcodage terminé pour la piste {}", track_number);
                ml_clone.quit();
            }
            MessageView::Error(err) => {
                eprintln!("Erreur lors du transcodage: {} ({:?})", err.error(), err.debug());
                ml_clone.quit();
            }
            MessageView::StateChanged(_) => {
                // Log optionnel des changements d'état
            }
            _ => {}
        }
        ControlFlow::Continue
    })?;

    // Démarrer le transcodage
    pipeline.set_state(State::Playing)?;
    main_loop.run();
    pipeline.set_state(State::Null)?;
    
    Ok(())
}

fn apply_metadata_to_pipeline(
    pipeline: &Pipeline,
    track: &TrackDetails,
    album: &AlbumDetails,
) -> Result<(), Box<dyn std::error::Error>> {
    use gstreamer::tags;
    
    let mut tag_list = gstreamer::TagList::new();
    
    // Titre de la piste
    tag_list.get_mut().unwrap().add::<tags::Title>(&track.title.as_str(), gstreamer::TagMergeMode::Replace);
    
    // Artiste
    if let Some(ref artist) = track.artist {
        tag_list.get_mut().unwrap().add::<tags::Artist>(&artist.as_str(), gstreamer::TagMergeMode::Replace);
    }
    
    // Album
    tag_list.get_mut().unwrap().add::<tags::Album>(&album.title.as_str(), gstreamer::TagMergeMode::Replace);
    
    // Numéro de piste
    tag_list.get_mut().unwrap().add::<tags::TrackNumber>(&track.number, gstreamer::TagMergeMode::Replace);
    
    // Date de sortie (ignorer pour l'instant car le type est complexe)
    // if let Some(ref date) = album.release_date {
    //     tag_list.get_mut().unwrap().add::<tags::Date>(date, gstreamer::TagMergeMode::Replace);
    // }
    
    // Envoyer les tags au pipeline
    let tag_event = gstreamer::event::Tag::new(tag_list);
    pipeline.send_event(tag_event);
    
    Ok(())
}

fn transcode_all_tracks(disc: &DiscId, album: &AlbumDetails, format: AudioFormat) -> Result<(), Box<dyn std::error::Error>> {
    std::fs::create_dir_all("output")?;
    
    println!("Début du transcodage de l'album : {}", album.title);
    println!("Format d'encodage : {}", format.name());
    println!("Nombre de pistes : {}", album.tracks.len());
    
    for track in &album.tracks {
        let filename = format!("output/{}", create_output_filename(track, album, format));
        
        match transcode_cd_track(disc, track, album, &filename, format) {
            Ok(()) => {
                println!("✓ Piste {} transcodée avec succès", track.number);
            }
            Err(e) => {
                eprintln!("✗ Erreur lors du transcodage de la piste {}: {}", track.number, e);
            }
        }
    }
    
    Ok(())
}

fn get_available_formats() -> Vec<AudioFormat> {
    vec![
        AudioFormat::Opus,
        AudioFormat::Vorbis,
        AudioFormat::Flac,
        AudioFormat::Mp3,
        AudioFormat::Aac,
        AudioFormat::Wavpack,
    ]
}

fn check_format_support(format: AudioFormat) -> bool {
    // Vérifier si encodebin existe
    let encodebin = match ElementFactory::make("encodebin").build() {
        Ok(e) => e,
        Err(_) => {
            eprintln!("   ⚠ encodebin non disponible pour {}", format.name());
            return false;
        }
    };
    
    let profile = create_encoding_profile(format);
    encodebin.set_property("profile", &profile);
    
    // Essayer de demander un pad audio
    let pad = encodebin.request_pad_simple("audio_%u");
    let supported = pad.is_some();
    
    if !supported {
        eprintln!("   ✗ {} - plugins manquants", format.name());
    }
    
    // Nettoyer
    if let Some(pad) = pad {
        encodebin.release_request_pad(&pad);
    }
    
    supported
}

fn get_supported_formats() -> Vec<AudioFormat> {
    get_available_formats()
        .into_iter()
        .filter(|&format| check_format_support(format))
        .collect()
}

fn select_format() -> AudioFormat {
    use std::io::{self, Write};
    
    println!("\n=== Sélection du format d'encodage ===");
    println!("Vérification des formats supportés...");
    
    let formats = get_supported_formats();
    
    if formats.is_empty() {
        eprintln!("❌ Aucun format d'encodage supporté trouvé !");
        eprintln!("Veuillez installer les plugins GStreamer nécessaires.");
        std::process::exit(1);
    }
    
    println!("\nFormats disponibles :");
    for (i, format) in formats.iter().enumerate() {
        let lossless = if format.is_lossless() { " (sans perte)" } else { "" };
        println!("{}. {}{}", i + 1, format.name(), lossless);
    }
    
    loop {
        print!("\nChoisissez un format (1-{}) [défaut: 1]: ", formats.len());
        io::stdout().flush().unwrap();
        
        let mut input = String::new();
        io::stdin().read_line(&mut input).unwrap();
        let input = input.trim();
        
        // Si l'utilisateur appuie juste sur Entrée, utiliser le format par défaut
        if input.is_empty() {
            println!("Format sélectionné : {}", formats[0].name());
            return formats[0];
        }
        
        // Essayer de parser le choix de l'utilisateur
        match input.parse::<usize>() {
            Ok(choice) if choice >= 1 && choice <= formats.len() => {
                let selected = formats[choice - 1];
                println!("Format sélectionné : {}", selected.name());
                return selected;
            }
            _ => {
                println!("❌ Choix invalide. Veuillez entrer un nombre entre 1 et {}", formats.len());
            }
        }
    }
}

fn main() {
    gstreamer::init().unwrap();
    let version = gstreamer::version_string();
    println!("{}", version);

    let disc = DiscId::read_features(None, Features::all()).expect("Reading disc failed");

    print_disc_info(&disc);

    // Sélectionner le format d'encodage
    let audio_format = select_format();

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
                
                // Sélectionner l'album à transcoder
                let selected_album = if albums.len() == 1 {
                    println!("\nTranscodage de l'album unique trouvé...");
                    &albums[0]
                } else {
                    println!("\nPlusieurs albums trouvés.");
                    select_album(&albums)
                };
                
                if let Err(e) = transcode_all_tracks(&disc, selected_album, audio_format) {
                    eprintln!("Erreur lors du transcodage : {}", e);
                }
            }
        }
        Err(e) => {
            println!("Error fetching album metadata: {}", e);
        }
    }
}

fn select_album(albums: &[AlbumDetails]) -> &AlbumDetails {
    use std::io::{self, Write};
    
    println!("\n=== Sélection de l'album ===");
    for (i, album) in albums.iter().enumerate() {
        print!("{}. {} - {}", i + 1, album.title, 
               album.artist.as_ref().unwrap_or(&"Unknown Artist".to_string()));
        if let Some(ref date) = album.release_date {
            print!(" ({})", date);
        }
        if let Some(ref country) = album.country {
            print!(" [{}]", country);
        }
        println!();
    }
    
    loop {
        print!("\nChoisissez un album (1-{}) [défaut: 1]: ", albums.len());
        io::stdout().flush().unwrap();
        
        let mut input = String::new();
        io::stdin().read_line(&mut input).unwrap();
        let input = input.trim();
        
        // Si l'utilisateur appuie juste sur Entrée, utiliser le premier album
        if input.is_empty() {
            println!("Album sélectionné : {}", albums[0].title);
            return &albums[0];
        }
        
        // Essayer de parser le choix de l'utilisateur
        match input.parse::<usize>() {
            Ok(choice) if choice >= 1 && choice <= albums.len() => {
                let selected = &albums[choice - 1];
                println!("Album sélectionné : {}", selected.title);
                return selected;
            }
            _ => {
                println!("❌ Choix invalide. Veuillez entrer un nombre entre 1 et {}", albums.len());
            }
        }
    }
}
