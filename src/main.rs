use adw::prelude::*;
use adw::{ActionRow, Application, ApplicationWindow, HeaderBar, SwitchRow, AboutDialog};
use gtk::{gio, Button, FileDialog, FileFilter, Box as GtkBox, ListBox, Orientation, SelectionMode};
use gtk::glib;
use std::fs;
use std::path::{Path, PathBuf};
use std::rc::Rc; // <-- important
use glib::{ControlFlow, MainLoop, object::ObjectExt};
use gstreamer::{
    Element, ElementFactory, MessageView, PadLinkSuccess, Pipeline, State, prelude::*,
};

trait AudioEncoder {
    fn new() -> Self;
    fn get_encoder(&self) -> &Element;
    fn get_muxer(&self) -> &Element;
}

struct OpusEncoder {
    encoder: Element,
    muxer: Element,
}

impl AudioEncoder for OpusEncoder {
    fn new() -> Self {
        let encoder = ElementFactory::make("opusenc").build().unwrap();
        encoder.set_property("bitrate", &(100_i32 * 1000_i32));
        encoder.set_property_from_str("bitrate-type", "vbr");
        encoder.set_property_from_str("bandwidth", "auto");
        let muxer = ElementFactory::make("oggmux").build().unwrap();
        OpusEncoder { encoder, muxer }
    }

    fn get_encoder(&self) -> &Element {
        &self.encoder
    }

    fn get_muxer(&self) -> &Element {
        &self.muxer
    }
}

fn is_audio_extension(ext: &str) -> bool {
    matches!(
        ext.to_ascii_lowercase().as_str(),
        "mp3" | "wav" | "flac" | "ogg" | "m4a" | "aac" | "opus"
    )
}

fn add_file_to_list(list: &ListBox, path: &Path) {
    let title = path
        .file_name()
        .map(|s| s.to_string_lossy().into_owned())
        .unwrap_or_else(|| path.display().to_string());

    let row = ActionRow::builder()
        .activatable(true)
        .title(&title)
        .build();

    let path_buf = path.to_path_buf();
    row.connect_activated(move |_| {
        eprintln!("Ligne activée : {}", path_buf.display());
    });

    list.append(&row);
}

/// Parcourt `folder` et ajoute à `list` tous les fichiers dont l'extension
/// est reconnue comme audio.
fn scan_folder_and_add_audio(list: &ListBox, folder: &Path) {
    if let Ok(entries) = fs::read_dir(folder) {
        for entry in entries.flatten() {
            let p = entry.path();
            if p.is_file() {
                if let Some(ext) = p.extension().and_then(|e| e.to_str()) {
                    if is_audio_extension(ext) {
                        add_file_to_list(list, &p);
                    }
                }
            }
        }
    } else {
        eprintln!("Impossible de lire le dossier: {}", folder.display());
    }
}

/// Création d'un bouton icône avec tooltip optionnel
fn create_icon_button(icon_name: &str, tooltip: Option<&str>) -> Button {
    let btn = Button::builder().icon_name(icon_name).build();
    if let Some(t) = tooltip {
        btn.set_tooltip_text(Some(t));
    }
    btn
}

/// Création d'un bouton avec label
fn create_label_button(label: &str) -> Button {
    Button::builder().label(label).build()
}

/// Configure un bouton pour ouvrir un dialogue qui renvoie un fichier.
/// `on_file_selected` est un Rc<dyn Fn(PathBuf)> clonable.
fn setup_file_button(
    file_btn: &Button,
    parent: &ApplicationWindow,
    filters_model: &gio::ListStore,
    on_file_selected: Rc<dyn Fn(PathBuf)>,
) {
    let parent = parent.clone();
    let filters = filters_model.clone();
    let cb = on_file_selected.clone();

    file_btn.connect_clicked(move |_| {
        let dlg = FileDialog::new();
        dlg.set_title("Ouvrir un fichier audio");
        dlg.set_filters(Some(&filters));
        let parent = parent.clone();
        let cb_inner = cb.clone();
        dlg.open(Some(&parent), None::<&gio::Cancellable>, move |res: Result<gio::File, glib::Error>| {
            match res {
                Ok(gfile) => {
                    if let Some(path) = gfile.path() {
                        eprintln!("Fichier audio choisi : {}", path.display());
                        (cb_inner)(path);
                    } else {
                        eprintln!("Fichier choisi (pas de path) : {:?}", gfile);
                    }
                }
                Err(err) => eprintln!("Aucun fichier sélectionné ou erreur: {}", err),
            }
        });
    });
}

/// Configure un bouton pour sélectionner un dossier.
/// `on_folder_selected` est un Rc<dyn Fn(PathBuf)> clonable.
fn setup_folder_button(
    folder_btn: &Button,
    parent: &ApplicationWindow,
    on_folder_selected: Rc<dyn Fn(PathBuf)>,
) {
    let parent = parent.clone();
    let cb = on_folder_selected.clone();

    folder_btn.connect_clicked(move |_| {
        let dlg = FileDialog::new();
        dlg.set_title("Choisir un dossier");
        let parent = parent.clone();
        let cb_inner = cb.clone();
        dlg.select_folder(Some(&parent), None::<&gio::Cancellable>, move |res: Result<gio::File, glib::Error>| {
            match res {
                Ok(gfile) => {
                    if let Some(path) = gfile.path() {
                        eprintln!("Dossier choisi : {}", path.display());
                        (cb_inner)(path);
                    } else {
                        eprintln!("Dossier choisi (pas de path) : {:?}", gfile);
                    }
                }
                Err(err) => eprintln!("Annulé / erreur sélection dossier: {}", err),
            }
        });
    });
}

fn main() {
    gstreamer::init().unwrap();
    let version = gstreamer::version_string();
    println!("{}", version);

    let pipeline = Pipeline::new();
    let source = ElementFactory::make_with_name("filesrc", Some("src")).unwrap();
    source.set_property_from_str("location", "FallingSky.mp3");
    let decodebin = ElementFactory::make_with_name("decodebin", Some("decoder")).unwrap();
    let audiorate = ElementFactory::make("audiorate").build().unwrap();
    let audioconvert = ElementFactory::make("audioconvert").build().unwrap();
    let audioresample = ElementFactory::make("audioresample").build().unwrap();

    // Choix de l'encodeur avec un énum
    let encoder = OpusEncoder::new();

    let sink = ElementFactory::make("filesink").build().unwrap();
    sink.set_property_from_str("location", "FallingSky.opus");

    pipeline
        .add_many(&[
            &source,
            &decodebin,
            &audiorate,
            &audioconvert,
            &audioresample,
            encoder.get_encoder(),
            encoder.get_muxer(),
            &sink,
        ])
        .unwrap();

    source.link(&decodebin).unwrap();
    audiorate.link(&audioconvert).unwrap();
    audioconvert.link(&audioresample).unwrap();
    audioresample.link(encoder.get_encoder()).unwrap();
    encoder.get_encoder().link(encoder.get_muxer()).unwrap();
    encoder.get_muxer().link(&sink).unwrap();

    let audiorate_clone = audiorate.clone();
    decodebin.connect_pad_added(move |_dbin, src_pad| {
        println!("decodebin pad-added: {}", src_pad.name());
        if let Some(caps) = src_pad.current_caps() {
            if let Some(structure) = caps.structure(0) {
                let media_type = structure.name();
                println!("Pad caps: {}", media_type);
                if !media_type.starts_with("audio/") && media_type != "audio/x-raw" {
                    println!("Ignoring non-audio pad: {}", media_type);
                    return;
                }
            }
        }
        let sink_pad = audiorate_clone
            .static_pad("sink")
            .expect("audiorate has no sink pad");
        match src_pad.link(&sink_pad) {
            Ok(PadLinkSuccess) => {
                println!("Linked decodebin -> audiorate");
            }
            Err(err) => {
                eprintln!("Failed to link decodebin pad to audiorate: {:?}", err);
            }
        }
    });

    let bus = pipeline
        .bus()
        .expect("Pipeline without bus — this should not happen");
    let main_loop = MainLoop::new(None, false);
    let ml_clone = main_loop.clone();
    let _bus_watch = bus
        .add_watch(move |_bus, msg| {
            match msg.view() {
                MessageView::Eos(_) => {
                    println!("EOS received — stopping main loop");
                    ml_clone.quit();
                }
                MessageView::Error(err) => {
                    let src = err
                        .src()
                        .map(|s| s.path_string())
                        .unwrap_or_else(|| glib::GString::from("unknown"));
                    eprintln!("Error from {}: {} ({:?})", src, err.error(), err.debug());
                    ml_clone.quit();
                }
                MessageView::StateChanged(_s) => {
                    // optional logging
                }
                _ => {}
            }
            ControlFlow::Continue
        })
        .expect("Failed to add bus watch");

    pipeline
        .set_state(State::Playing)
        .expect("Unable to set the pipeline to `Playing` state");
    println!("Running main loop...");
    main_loop.run();
    pipeline
        .set_state(State::Null)
        .expect("Failed to set pipeline to Null state");
    println!("Done.");
    
    let application = Application::builder()
        .application_id("com.example.FirstAdwaitaApp")
        .build();

    application.connect_activate(|app| {
        // --- boutons ---
        let convert_btn = create_label_button("Convertir");
        let file_btn = create_icon_button("document-open-symbolic", Some("Choisir un fichier audio"));
        let folder_btn = create_icon_button("folder-symbolic", Some("Choisir un dossier"));
        let prefs_btn = create_icon_button("preferences-system-symbolic", Some("Préférences"));
        let about_btn = create_icon_button("help-about-symbolic", Some("À propos"));

        // --- HeaderBar ---
        let header = HeaderBar::new();
        header.pack_start(&convert_btn);
        header.pack_start(&file_btn);
        header.pack_start(&folder_btn);
        header.pack_end(&prefs_btn);
        header.pack_end(&about_btn);

        // --- contenu (LISTE sans CSS personnalisé) ---
        let example_row = ActionRow::builder().activatable(true).title("Click me").build();
        example_row.connect_activated(|_| eprintln!("Clicked!"));

        let switch = SwitchRow::new();
        switch.set_title("Switch me");

        let list = ListBox::builder()
            .margin_top(16)
            .margin_end(16)
            .margin_bottom(16)
            .margin_start(16)
            .selection_mode(SelectionMode::None)
            .css_classes(vec![String::from("boxed-list")])
            .build();

        let content = GtkBox::new(Orientation::Vertical, 0);
        content.append(&header);
        content.append(&list);

        // --- fenêtre ---
        let window = ApplicationWindow::builder()
            .application(app)
            .title("First App")
            .default_width(700)
            .default_height(420)
            .content(&content)
            .build();

        // --- filtre audio (réutilisable) ---
        let audio_filter = FileFilter::new();
        audio_filter.set_name(Some("Fichiers audio"));
        audio_filter.add_mime_type("audio/*");

        let filters_model = gio::ListStore::new::<FileFilter>();
        filters_model.append(&audio_filter);

        // --- handlers simplifiés et sans duplication ---
        {
            // file button: ajoute 1 fichier choisi à la liste
            let list_for_file = list.clone();
            let cb = Rc::new(move |path: PathBuf| {
                add_file_to_list(&list_for_file, &path);
            }) as Rc<dyn Fn(PathBuf)>;
            setup_file_button(&file_btn, &window, &filters_model, cb);
        }

        {
            // folder button: scan dossier et ajoute tous les audios
            let list_for_folder = list.clone();
            let cb = Rc::new(move |path: PathBuf| {
                scan_folder_and_add_audio(&list_for_folder, &path);
            }) as Rc<dyn Fn(PathBuf)>;
            setup_folder_button(&folder_btn, &window, cb);
        }

        convert_btn.connect_clicked(|_| eprintln!("Convertir cliqué"));
        prefs_btn.connect_clicked(|_| eprintln!("Préférences cliqué"));

        {
            // about dialog
            let win_for_about = window.clone();
            about_btn.connect_clicked(move |_| {
                let about = AboutDialog::new();
                about.set_application_name("First App");
                about.set_version("0.1");
                about.set_comments("Application de conversion audio — exemple");
                about.present(Some(&win_for_about));
            });
        }

        window.present();
    });

    application.run();
}
