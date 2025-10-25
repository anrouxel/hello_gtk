use crate::audio::{AudioFormat, FormatValidator};
use crate::metadata::AlbumDetails;
use std::io::{self, Write};

pub struct UserInterface;

impl UserInterface {
    pub fn select_mode() -> u8 {
        println!("\n=== Mode de fonctionnement ===");
        println!("1. Ripper et transcoder un CD audio depuis un disque physique");
        println!("2. Lire des fichiers audio depuis un dossier");
        println!("3. Lire les pistes d'un CD audio directement");
        
        loop {
            print!("\nChoisissez un mode (1-3) [défaut: 1]: ");
            io::stdout().flush().unwrap();
            
            let mut input = String::new();
            io::stdin().read_line(&mut input).unwrap();
            let input = input.trim();
            
            if input.is_empty() {
                return 1;
            }
            
            match input.parse::<u8>() {
                Ok(choice) if (1..=3).contains(&choice) => {
                    return choice;
                }
                _ => {
                    println!("❌ Choix invalide. Veuillez entrer 1, 2 ou 3");
                }
            }
        }
    }

    pub fn select_format() -> AudioFormat {
        println!("\n=== Sélection du format d'encodage ===");
        println!("Vérification des formats supportés...");
        
        let formats = FormatValidator::get_supported_formats();
        
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
            
            if input.is_empty() {
                println!("Format sélectionné : {}", formats[0].name());
                return formats[0];
            }
            
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

    pub fn select_album(albums: &[AlbumDetails]) -> &AlbumDetails {
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
            
            if input.is_empty() {
                println!("Album sélectionné : {}", albums[0].title);
                return &albums[0];
            }
            
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

    pub fn select_directory() -> String {
        println!("\n=== Sélection du dossier ===");
        print!("Entrez le chemin du dossier (ou appuyez sur Entrée pour 'output/'): ");
        io::stdout().flush().unwrap();
        
        let mut input = String::new();
        io::stdin().read_line(&mut input).unwrap();
        let input = input.trim();
        
        if input.is_empty() {
            "output".to_string()
        } else {
            input.to_string()
        }
    }

    pub fn list_audio_files(directory: &str) -> Result<Vec<String>, Box<dyn std::error::Error>> {
        let mut files = Vec::new();
        let audio_extensions = vec!["opus", "ogg", "flac", "mp3", "m4a", "wv", "wav"];
        
        for entry in std::fs::read_dir(directory)? {
            let entry = entry?;
            let path = entry.path();
            
            if path.is_file() {
                if let Some(extension) = path.extension() {
                    if audio_extensions.contains(&extension.to_str().unwrap_or("")) {
                        if let Some(file_name) = path.file_name() {
                            files.push(file_name.to_str().unwrap_or("").to_string());
                        }
                    }
                }
            }
        }
        
        files.sort();
        Ok(files)
    }

    pub fn ask_continue(message: &str) -> bool {
        print!("\n{} (o/N): ", message);
        io::stdout().flush().unwrap();
        
        let mut input = String::new();
        io::stdin().read_line(&mut input).unwrap();
        let input = input.trim().to_lowercase();
        
        matches!(input.as_str(), "o" | "oui" | "y" | "yes")
    }

    pub fn prompt_choice(prompt: &str, min: usize, max: usize) -> Option<usize> {
        print!("\n{}: ", prompt);
        io::stdout().flush().unwrap();
        
        let mut input = String::new();
        io::stdin().read_line(&mut input).unwrap();
        let input = input.trim();
        
        match input.parse::<usize>() {
            Ok(0) => None,
            Ok(choice) if choice >= min && choice <= max => Some(choice),
            _ => {
                println!("❌ Choix invalide");
                None
            }
        }
    }
}
