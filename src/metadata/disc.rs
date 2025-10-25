use discid::DiscId;

#[derive(Debug, Clone)]
pub struct DiscDetails {
    pub id: String,
    pub mcn: Option<String>,
    pub url: String,
    pub release_ids: Vec<String>,
}

impl DiscDetails {
    pub fn from_disc(disc: &DiscId) -> Self {
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

    pub fn print_disc_info(disc: &DiscId) {
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
}
