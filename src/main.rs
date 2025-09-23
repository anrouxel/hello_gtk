use adw::prelude::*;
use adw::{ActionRow, Application, ApplicationWindow, HeaderBar, SwitchRow, AboutDialog};
use gtk::{gio, Button, FileDialog, FileFilter, Box as GtkBox, ListBox, Orientation, SelectionMode};
use glib;

fn main() {
    adw::init().expect("Failed to init adw");

    let application = Application::builder()
        .application_id("com.example.FirstAdwaitaApp")
        .build();

    application.connect_activate(|app| {
        // --- boutons icônes (boutons sans texte) ---
        let convert_btn = Button::builder()
            .icon_name("media-playback-start-symbolic")
            .build();
        convert_btn.set_tooltip_text(Some("Convertir"));

        let file_btn = Button::builder()
            .icon_name("document-open-symbolic")
            .build();
        file_btn.set_tooltip_text(Some("Choisir un fichier audio"));

        let folder_btn = Button::builder()
            .icon_name("folder-symbolic")
            .build();
        folder_btn.set_tooltip_text(Some("Choisir un dossier"));

        let prefs_btn = Button::builder()
            .icon_name("preferences-system-symbolic")
            .build();
        prefs_btn.set_tooltip_text(Some("Préférences"));

        let about_btn = Button::builder()
            .icon_name("help-about-symbolic")
            .build();
        about_btn.set_tooltip_text(Some("À propos"));

        // --- HeaderBar ---
        let header = HeaderBar::new();
        header.pack_start(&convert_btn);
        header.pack_start(&file_btn);
        header.pack_start(&folder_btn);
        header.pack_end(&prefs_btn);
        header.pack_end(&about_btn);

        // --- contenu simplifié ---
        let row = ActionRow::builder().activatable(true).title("Click me").build();
        row.connect_activated(|_| eprintln!("Clicked!"));
        let switch = SwitchRow::new();
        switch.set_title("Switch me");
        let list = ListBox::builder()
            .margin_top(32)
            .margin_end(32)
            .margin_bottom(32)
            .margin_start(32)
            .selection_mode(SelectionMode::None)
            .css_classes(vec![String::from("boxed-list")])
            .build();
        list.append(&row);
        list.append(&switch);

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

        // --- filtre audio ---
        let audio_filter = FileFilter::new();
        audio_filter.set_name(Some("Fichiers audio"));
        audio_filter.add_mime_type("audio/*");

        let filters_model = gio::ListStore::new::<FileFilter>();
        filters_model.append(&audio_filter);

        // --- handlers ---
        let win_for_file = window.clone();
        let filters_model_for_file = filters_model.clone();
        file_btn.connect_clicked(move |_| {
            let dlg = FileDialog::new();
            dlg.set_title("Ouvrir un fichier audio");
            dlg.set_filters(Some(&filters_model_for_file));

            dlg.open(Some(&win_for_file), None::<&gio::Cancellable>, move |res: Result<gio::File, glib::Error>| {
                match res {
                    Ok(gfile) => {
                        if let Some(path) = gfile.path() {
                            eprintln!("Fichier audio choisi : {}", path.display());
                        } else {
                            eprintln!("Fichier choisi (pas de path) : {:?}", gfile);
                        }
                    }
                    Err(err) => eprintln!("Aucun fichier sélectionné ou erreur: {}", err),
                }
            });
        });

        let win_for_folder = window.clone();
        folder_btn.connect_clicked(move |_| {
            let dlg = FileDialog::new();
            dlg.set_title("Choisir un dossier");
            dlg.select_folder(Some(&win_for_folder), None::<&gio::Cancellable>, move |res: Result<gio::File, glib::Error>| {
                match res {
                    Ok(gfile) => {
                        if let Some(path) = gfile.path() {
                            eprintln!("Dossier choisi : {}", path.display());
                        }
                    }
                    Err(err) => eprintln!("Annulé / erreur sélection dossier: {}", err),
                }
            });
        });

        convert_btn.connect_clicked(|_| eprintln!("Convertir cliqué"));
        prefs_btn.connect_clicked(|_| eprintln!("Préférences cliqué"));

        let win_for_about = window.clone();
        about_btn.connect_clicked(move |_| {
            let about = AboutDialog::new();
            about.set_application_name("First App");
            about.set_version("0.1");
            about.set_comments("Application de conversion audio — exemple");
            about.present(Some(&win_for_about));
        });

        window.present();
    });

    application.run();
}
