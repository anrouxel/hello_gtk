#[derive(Debug, Clone)]
pub struct TrackDetails {
    pub number: u32,
    pub title: String,
    pub duration: Option<u32>,
    pub artist: Option<String>,
    pub artist_sortname: Option<String>,
    pub artist_id: Option<String>,
    pub track_id: Option<String>,
    pub composer: Option<String>,
    pub composer_sortname: Option<String>,
}

impl TrackDetails {
    pub fn duration_string(&self) -> Option<String> {
        self.duration.map(|duration| {
            let seconds = duration / 1000;
            format!("{}:{:02}", seconds / 60, seconds % 60)
        })
    }
}
