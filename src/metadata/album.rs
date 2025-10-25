use super::TrackDetails;

#[derive(Debug, Clone)]
pub struct AlbumDetails {
    pub album_id: String,
    pub title: String,
    pub artist: Option<String>,
    pub artist_sortname: Option<String>,
    pub artist_id: Option<String>,
    pub release_date: Option<String>,
    pub country: Option<String>,
    pub disc_number: Option<u32>,
    pub disc_count: Option<u32>,
    pub barcode: Option<String>,
    pub tracks: Vec<TrackDetails>,
    pub composer: Option<String>,
    pub composer_sortname: Option<String>,
}

impl AlbumDetails {
    pub fn display_info(&self) {
        println!("Title: {}", self.title);
        if let Some(ref artist) = self.artist {
            println!("Artist: {}", artist);
        }
        if let Some(ref date) = self.release_date {
            println!("Release Date: {}", date);
        }
        if let Some(ref country) = self.country {
            println!("Country: {}", country);
        }
        if let Some(ref barcode) = self.barcode {
            println!("Barcode: {}", barcode);
        }
        
        println!("Tracks ({}):", self.tracks.len());
        for track in &self.tracks {
            print!("  {}: {}", track.number, track.title);
            if let Some(duration_str) = track.duration_string() {
                print!(" ({})", duration_str);
            }
            println!();
        }
    }
}
