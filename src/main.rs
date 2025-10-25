mod audio;
mod metadata;
mod encoder;
mod player;
mod ui;
mod utils;

use discid::{DiscId, Features};
use glib::MainLoop;
use metadata::{DiscDetails, MusicBrainzClient};
use encoder::TranscodeManager;
use player::{AudioPlayer, CdTrackPlayer, FilePlayer};
use ui::UserInterface;
use std::error::Error;

fn rip_cd_mode() -> Result<(), Box<dyn Error>> {
    let disc = DiscId::read_features(None, Features::all())
        .expect("Reading disc failed");
    
    DiscDetails::print_disc_info(&disc);
    
    let audio_format = UserInterface::select_format();
    
    println!("\n=== MusicBrainz Metadata ===");
    match MusicBrainzClient::list_albums(&disc) {
        Ok(albums) => {
            if albums.is_empty() {
                println!("No album metadata found");
            } else {
                for (i, album) in albums.iter().enumerate() {
                    println!("\n--- Album {} ---", i + 1);
                    album.display_info();
                }
                
                let selected_album = if albums.len() == 1 {
                    println!("\nTranscodage de l'album unique trouvé...");
                    &albums[0]
                } else {
                    println!("\nPlusieurs albums trouvés.");
                    UserInterface::select_album(&albums)
                };
                
                let manager = TranscodeManager::new();
                manager.transcode_all_tracks(&disc, selected_album, audio_format)?;
            }
        }
        Err(e) => {
            println!("Error fetching album metadata: {}", e);
        }
    }
    Ok(())
}

fn play_files_mode() -> Result<(), Box<dyn Error>> {
    println!("\n=== Lecteur de fichiers audio ===");
    
    let directory = UserInterface::select_directory();
    println!("Recherche de fichiers dans le dossier '{}'...\n", directory);
    
    let files = UserInterface::list_audio_files(&directory)?;
    
    if files.is_empty() {
        println!("❌ Aucun fichier audio trouvé dans le dossier '{}'", directory);
        return Ok(());
    }
    
    let main_loop = MainLoop::new(None, false);
    let ml_for_ctrlc = main_loop.clone();
    
    ctrlc::set_handler(move || {
        println!("\n⏹ Arrêt de la lecture...");
        ml_for_ctrlc.quit();
    })?;
    
    println!("Fichiers disponibles ({}) :", files.len());
    for (i, file) in files.iter().enumerate() {
        println!("{}. {}", i + 1, file);
    }
    
    loop {
        if let Some(choice) = UserInterface::prompt_choice(
            &format!("Choisissez un fichier à lire (1-{}, 0 pour quitter)", files.len()),
            1,
            files.len(),
        ) {
            let selected_file = &files[choice - 1];
            let file_path = format!("{}/{}", directory, selected_file);
            
            let player = FilePlayer::new(file_path);
            match player.play(&main_loop) {
                Ok(()) => println!("\n✓ Lecture terminée avec succès"),
                Err(e) => eprintln!("\n❌ Erreur lors de la lecture : {}", e),
            }
            
            if !UserInterface::ask_continue("Lire un autre fichier ?") {
                println!("Au revoir !");
                return Ok(());
            }
            
            println!("\nFichiers disponibles ({}) :", files.len());
            for (i, file) in files.iter().enumerate() {
                println!("{}. {}", i + 1, file);
            }
        } else {
            println!("Au revoir !");
            return Ok(());
        }
    }
}

fn play_cd_mode() -> Result<(), Box<dyn Error>> {
    println!("\n=== Lecteur de CD Audio ===");
    println!("Lecture des informations du disque...\n");
    
    let disc = DiscId::read_features(None, Features::all())
        .map_err(|e| {
            eprintln!("❌ Erreur lors de la lecture du disque: {}", e);
            eprintln!("Assurez-vous qu'un CD audio est inséré dans le lecteur.");
            e
        })?;
    
    DiscDetails::print_disc_info(&disc);
    
    println!("\n=== Récupération des métadonnées ===");
    let albums = match MusicBrainzClient::list_albums(&disc) {
        Ok(albums) if !albums.is_empty() => {
            let album = if albums.len() == 1 {
                &albums[0]
            } else {
                UserInterface::select_album(&albums)
            };
            
            println!("\nAlbum: {}", album.title);
            if let Some(ref artist) = album.artist {
                println!("Artiste: {}", artist);
            }
            println!("\nPistes disponibles ({}) :", album.tracks.len());
            for track in &album.tracks {
                print!("{}. {}", track.number, track.title);
                if let Some(duration_str) = track.duration_string() {
                    print!(" ({})", duration_str);
                }
                println!();
            }
            Some(album.clone())
        }
        Ok(_) => {
            println!("Aucune métadonnée trouvée sur MusicBrainz");
            println!("\nPistes disponibles ({}) :", disc.last_track_num() - disc.first_track_num() + 1);
            for i in disc.first_track_num()..=disc.last_track_num() {
                println!("{}. Piste {}", i, i);
            }
            None
        }
        Err(e) => {
            println!("Erreur lors de la récupération des métadonnées: {}", e);
            println!("\nPistes disponibles ({}) :", disc.last_track_num() - disc.first_track_num() + 1);
            for i in disc.first_track_num()..=disc.last_track_num() {
                println!("{}. Piste {}", i, i);
            }
            None
        }
    };
    
    let main_loop = MainLoop::new(None, false);
    let ml_for_ctrlc = main_loop.clone();
    
    ctrlc::set_handler(move || {
        println!("\n⏹ Arrêt de la lecture...");
        ml_for_ctrlc.quit();
    })?;
    
    let first_track = disc.first_track_num() as u32;
    let last_track = disc.last_track_num() as u32;
    
    loop {
        if let Some(choice) = UserInterface::prompt_choice(
            &format!("Choisissez une piste à lire ({}-{}, 0 pour quitter)", first_track, last_track),
            first_track as usize,
            last_track as usize,
        ) {
            if let Some(ref album) = albums {
                if let Some(track) = album.tracks.iter().find(|t| t.number == choice as u32) {
                    println!("\n🎵 Piste {}: {}", choice, track.title);
                    if let Some(ref artist) = track.artist {
                        println!("   Artiste: {}", artist);
                    }
                }
            }
            
            let player = CdTrackPlayer::new(choice as u32);
            match player.play(&main_loop) {
                Ok(()) => println!("\n✓ Lecture terminée avec succès"),
                Err(e) => eprintln!("\n❌ Erreur lors de la lecture : {}", e),
            }
            
            if !UserInterface::ask_continue("Lire une autre piste ?") {
                println!("Au revoir !");
                return Ok(());
            }
            
            if let Some(ref album) = albums {
                println!("\nPistes disponibles ({}) :", album.tracks.len());
                for track in &album.tracks {
                    print!("{}. {}", track.number, track.title);
                    if let Some(duration_str) = track.duration_string() {
                        print!(" ({})", duration_str);
                    }
                    println!();
                }
            } else {
                println!("\nPistes disponibles ({}) :", last_track - first_track + 1);
                for i in first_track..=last_track {
                    println!("{}. Piste {}", i, i);
                }
            }
        } else {
            println!("Au revoir !");
            return Ok(());
        }
    }
}

fn main() {
    gstreamer::init().unwrap();
    let version = gstreamer::version_string();
    println!("{}", version);
    
    let mode = UserInterface::select_mode();
    
    let result = match mode {
        1 => rip_cd_mode(),
        2 => play_files_mode(),
        3 => play_cd_mode(),
        _ => {
            eprintln!("Mode invalide");
            std::process::exit(1);
        }
    };
    
    if let Err(e) = result {
        eprintln!("Erreur : {}", e);
        std::process::exit(1);
    }
}
