use crate::audio::AudioFormat;
use crate::metadata::{AlbumDetails, TrackDetails};
use crate::utils::sanitize_filename;
use discid::DiscId;
use glib::{ControlFlow, MainLoop};
use gstreamer::{ElementFactory, MessageView, Pipeline, State, prelude::*};

pub trait AudioEncoder {
    fn transcode(
        &self,
        disc: &DiscId,
        track: &TrackDetails,
        album: &AlbumDetails,
        output_filename: &str,
        format: AudioFormat,
    ) -> Result<(), Box<dyn std::error::Error>>;
}

pub struct GStreamerEncoder;

impl GStreamerEncoder {
    pub fn new() -> Self {
        Self
    }

    fn apply_metadata(
        &self,
        pipeline: &Pipeline,
        track: &TrackDetails,
        album: &AlbumDetails,
    ) -> Result<(), Box<dyn std::error::Error>> {
        use gstreamer::tags;
        
        let mut tag_list = gstreamer::TagList::new();
        
        tag_list.get_mut().unwrap().add::<tags::Title>(&track.title.as_str(), gstreamer::TagMergeMode::Replace);
        
        if let Some(ref artist) = track.artist {
            tag_list.get_mut().unwrap().add::<tags::Artist>(&artist.as_str(), gstreamer::TagMergeMode::Replace);
        }
        
        tag_list.get_mut().unwrap().add::<tags::Album>(&album.title.as_str(), gstreamer::TagMergeMode::Replace);
        tag_list.get_mut().unwrap().add::<tags::TrackNumber>(&track.number, gstreamer::TagMergeMode::Replace);
        
        let tag_event = gstreamer::event::Tag::new(tag_list);
        pipeline.send_event(tag_event);
        
        Ok(())
    }

    pub fn create_output_filename(track: &TrackDetails, album: &AlbumDetails, format: AudioFormat) -> String {
        let track_num = format!("{:02}", track.number);
        let title = sanitize_filename(&track.title);
        let artist = sanitize_filename(track.artist.as_ref().unwrap_or(&"Unknown".to_string()));
        let album_title = sanitize_filename(&album.title);
        let extension = format.file_extension();
        
        format!("{} - {} - {} - {}.{}", track_num, artist, album_title, title, extension)
    }
}

impl AudioEncoder for GStreamerEncoder {
    fn transcode(
        &self,
        _disc: &DiscId,
        track: &TrackDetails,
        album: &AlbumDetails,
        output_filename: &str,
        format: AudioFormat,
    ) -> Result<(), Box<dyn std::error::Error>> {
        println!("Transcodage de la piste {} : {} (format: {})", track.number, track.title, format.name());
        
        let pipeline = Pipeline::new();
        
        let source = ElementFactory::make_with_name("cdparanoiasrc", Some("src")).unwrap();
        source.set_property("track", track.number as u32);
        
        let audiorate = ElementFactory::make("audiorate").build().unwrap();
        let audioconvert = ElementFactory::make("audioconvert").build().unwrap();
        let audioresample = ElementFactory::make("audioresample").build().unwrap();
        
        let encodebin = ElementFactory::make("encodebin").build().unwrap();
        let profile = format.create_encoding_profile();
        encodebin.set_property("profile", &profile);
        
        let sink = ElementFactory::make("filesink").build().unwrap();
        sink.set_property_from_str("location", output_filename);

        pipeline.add_many(&[
            &source,
            &audiorate,
            &audioconvert,
            &audioresample,
            &encodebin,
            &sink,
        ])?;

        source.link(&audiorate)?;
        audiorate.link(&audioconvert)?;
        audioconvert.link(&audioresample)?;
        
        let audio_pad = encodebin.request_pad_simple("audio_%u")
            .ok_or_else(|| format!("Impossible de créer un pad audio pour encodebin. Le format {} n'est peut-être pas supporté ou les plugins nécessaires ne sont pas installés.", format.name()))?;
        let audioresample_src_pad = audioresample.static_pad("src")
            .ok_or("Impossible d'obtenir le pad source d'audioresample")?;
        audioresample_src_pad.link(&audio_pad)?;
        
        encodebin.link(&sink)?;

        self.apply_metadata(&pipeline, track, album)?;

        let bus = pipeline.bus().expect("Pipeline without bus");
        let main_loop = MainLoop::new(None, false);
        let ml_clone = main_loop.clone();
        
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
                MessageView::StateChanged(_) => {}
                _ => {}
            }
            ControlFlow::Continue
        })?;

        pipeline.set_state(State::Playing)?;
        main_loop.run();
        pipeline.set_state(State::Null)?;
        
        Ok(())
    }
}
