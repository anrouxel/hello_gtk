use glib::{ControlFlow, MainLoop};
use gstreamer::{ElementFactory, MessageView, Pipeline, State, prelude::*};
use std::error::Error;

pub trait AudioPlayer {
    fn play(&self, ml: &MainLoop) -> Result<(), Box<dyn Error>>;
}

pub struct CdTrackPlayer {
    track_number: u32,
}

impl CdTrackPlayer {
    pub fn new(track_number: u32) -> Self {
        Self { track_number }
    }
}

impl AudioPlayer for CdTrackPlayer {
    fn play(&self, ml: &MainLoop) -> Result<(), Box<dyn Error>> {
        println!("\n🎵 Lecture de la piste {} depuis le CD...", self.track_number);
        
        let pipeline = Pipeline::new();

        let cdparanoiasrc = ElementFactory::make("cdiocddasrc").build()?;
        cdparanoiasrc.set_property("track", self.track_number);

        let queue = ElementFactory::make("queue").build()?;
        queue.set_property("max-size-buffers", 0u32);
        queue.set_property("max-size-time", 5_000_000_000u64);
        queue.set_property("max-size-bytes", 10_485_760u32);
        
        let audioconvert = ElementFactory::make("audioconvert").build()?;
        let audioresample = ElementFactory::make("audioresample").build()?;
        let audiosink = ElementFactory::make("autoaudiosink").build()?;
        
        pipeline.add_many(&[&cdparanoiasrc, &queue, &audioconvert, &audioresample, &audiosink])?;
        cdparanoiasrc.link(&queue)?;
        queue.link(&audioconvert)?;
        audioconvert.link(&audioresample)?;
        audioresample.link(&audiosink)?;
        
        let bus = pipeline.bus().expect("Pipeline without bus");
        let ml_clone = ml.clone();
        
        let pipeline_clone_for_watch = pipeline.clone();
        let track_number = self.track_number;
        
        let _bus_watch = bus.add_watch(move |_bus, msg| {
            match msg.view() {
                MessageView::Eos(_) => {
                    println!("\n✓ Lecture de la piste {} terminée", track_number);
                    ml_clone.quit();
                }
                MessageView::Error(err) => {
                    eprintln!("\n❌ Erreur lors de la lecture: {} ({:?})", err.error(), err.debug());
                    ml_clone.quit();
                }
                MessageView::StateChanged(state_changed) => {
                    if let Some(element) = state_changed.src().and_then(|s| s.downcast_ref::<Pipeline>()) {
                        if element == &pipeline_clone_for_watch {
                            let old = state_changed.old();
                            let new = state_changed.current();
                            if new == State::Playing && old != State::Playing {
                                println!("▶ Lecture en cours... (Ctrl+C pour arrêter)");
                            }
                        }
                    }
                }
                MessageView::Tag(tag_msg) => {
                    let tags = tag_msg.tags();
                    if let Some(title) = tags.get::<gstreamer::tags::Title>() {
                        println!("   Titre: {}", title.get());
                    }
                    if let Some(artist) = tags.get::<gstreamer::tags::Artist>() {
                        println!("   Artiste: {}", artist.get());
                    }
                    if let Some(album) = tags.get::<gstreamer::tags::Album>() {
                        println!("   Album: {}", album.get());
                    }
                }
                _ => {}
            }
            ControlFlow::Continue
        })?;
        
        pipeline.set_state(State::Playing)?;
        
        println!("\n⏸  Appuyez sur Ctrl+C pour arrêter la lecture");
        
        ml.run();
        pipeline.set_state(State::Null)?;
        
        Ok(())
    }
}

pub struct FilePlayer {
    file_path: String,
}

impl FilePlayer {
    pub fn new(file_path: String) -> Self {
        Self { file_path }
    }
}

impl AudioPlayer for FilePlayer {
    fn play(&self, ml: &MainLoop) -> Result<(), Box<dyn Error>> {
        println!("\n🎵 Lecture du fichier : {}", self.file_path);
        
        let pipeline = Pipeline::new();
        
        let filesrc = ElementFactory::make("filesrc").build()?;
        filesrc.set_property_from_str("location", &self.file_path);
        
        let decodebin = ElementFactory::make("decodebin3").build()?;
        let audioconvert = ElementFactory::make("audioconvert").build()?;
        let audioresample = ElementFactory::make("audioresample").build()?;
        let audiosink = ElementFactory::make("autoaudiosink").build()?;
        
        pipeline.add_many(&[&filesrc, &decodebin, &audioconvert, &audioresample, &audiosink])?;
        filesrc.link(&decodebin)?;
        audioconvert.link(&audioresample)?;
        audioresample.link(&audiosink)?;
        
        let audioconvert_clone = audioconvert.clone();
        decodebin.connect_pad_added(move |_element, src_pad| {
            let sink_pad = audioconvert_clone
                .static_pad("sink")
                .expect("Failed to get static sink pad from audioconvert");
            if sink_pad.is_linked() {
                return;
            }
            let _ = src_pad.link(&sink_pad);
        });
        
        let bus = pipeline.bus().expect("Pipeline without bus");
        let ml_clone = ml.clone();
        
        let pipeline_clone_for_watch = pipeline.clone();
        
        let _bus_watch = bus.add_watch(move |_bus, msg| {
            match msg.view() {
                MessageView::Eos(_) => {
                    println!("\n✓ Lecture terminée");
                    ml_clone.quit();
                }
                MessageView::Error(err) => {
                    eprintln!("\n❌ Erreur lors de la lecture: {} ({:?})", err.error(), err.debug());
                    ml_clone.quit();
                }
                MessageView::StateChanged(state_changed) => {
                    if let Some(element) = state_changed.src().and_then(|s| s.downcast_ref::<Pipeline>()) {
                        if element == &pipeline_clone_for_watch {
                            let old = state_changed.old();
                            let new = state_changed.current();
                            if new == State::Playing && old != State::Playing {
                                println!("▶ Lecture en cours... (Ctrl+C pour arrêter)");
                            }
                        }
                    }
                }
                MessageView::Tag(tag_msg) => {
                    let tags = tag_msg.tags();
                    if let Some(title) = tags.get::<gstreamer::tags::Title>() {
                        println!("   Titre: {}", title.get());
                    }
                    if let Some(artist) = tags.get::<gstreamer::tags::Artist>() {
                        println!("   Artiste: {}", artist.get());
                    }
                    if let Some(album) = tags.get::<gstreamer::tags::Album>() {
                        println!("   Album: {}", album.get());
                    }
                }
                _ => {}
            }
            ControlFlow::Continue
        })?;
        
        pipeline.set_state(State::Playing)?;
        
        println!("\n⏸  Appuyez sur Ctrl+C pour arrêter la lecture");
        
        ml.run();
        pipeline.set_state(State::Null)?;
        
        Ok(())
    }
}
