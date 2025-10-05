use adw::prelude::*;
use gtk::gio;

/// Composant HeaderBar
/// Responsable de la barre d'en-tête avec les boutons d'action
pub struct HeaderBar {
    widget: adw::HeaderBar,
    convert_btn: gtk::Button,
    open_file_btn: gtk::Button,
    open_folder_btn: gtk::Button,
    about_btn: gtk::Button,
    prefs_btn: gtk::Button,
}

impl HeaderBar {
    /// Crée une nouvelle barre d'en-tête
    pub fn new() -> Self {
        let widget = adw::HeaderBar::new();
        
        // Bouton Convertir
        let convert_btn = gtk::Button::builder()
            .label("Convertir")
            .tooltip_text("Convertir (Ctrl+R)")
            .build();
        
        // Bouton Ouvrir fichier
        let open_file_btn = gtk::Button::builder()
            .icon_name("document-open-symbolic")
            .tooltip_text("Choisir un fichier audio (Ctrl+O)")
            .build();
        
        // Bouton Ouvrir dossier
        let open_folder_btn = gtk::Button::builder()
            .icon_name("folder-symbolic")
            .tooltip_text("Choisir un dossier (Ctrl+F)")
            .build();
        
        // Bouton À propos
        let about_btn = gtk::Button::builder()
            .icon_name("help-about-symbolic")
            .tooltip_text("À propos")
            .build();
        
        // Bouton Préférences
        let prefs_btn = gtk::Button::builder()
            .icon_name("preferences-system-symbolic")
            .tooltip_text("Préférences")
            .build();
        
        widget.pack_start(&convert_btn);
        widget.pack_start(&open_file_btn);
        widget.pack_start(&open_folder_btn);
        widget.pack_end(&about_btn);
        widget.pack_end(&prefs_btn);
        
        Self {
            widget,
            convert_btn,
            open_file_btn,
            open_folder_btn,
            about_btn,
            prefs_btn,
        }
    }

    /// Retourne le widget GTK
    pub fn widget(&self) -> &adw::HeaderBar {
        &self.widget
    }

    /// Configure le callback pour le bouton Convertir
    pub fn on_convert<F>(&self, callback: F)
    where
        F: Fn() + 'static,
    {
        self.convert_btn.connect_clicked(move |_| {
            callback();
        });
    }

    /// Configure le callback pour le bouton Ouvrir fichier
    pub fn on_open_file<F>(&self, callback: F)
    where
        F: Fn() + 'static,
    {
        self.open_file_btn.connect_clicked(move |_| {
            callback();
        });
    }

    /// Configure le callback pour le bouton Ouvrir dossier
    pub fn on_open_folder<F>(&self, callback: F)
    where
        F: Fn() + 'static,
    {
        self.open_folder_btn.connect_clicked(move |_| {
            callback();
        });
    }

    /// Configure le callback pour le bouton À propos
    pub fn on_about<F>(&self, callback: F)
    where
        F: Fn() + 'static,
    {
        self.about_btn.connect_clicked(move |_| {
            callback();
        });
    }

    /// Configure le callback pour le bouton Préférences
    pub fn on_preferences<F>(&self, callback: F)
    where
        F: Fn() + 'static,
    {
        self.prefs_btn.connect_clicked(move |_| {
            callback();
        });
    }

    /// Configure les actions et raccourcis clavier
    pub fn setup_actions(&self, app: &adw::Application, window: &adw::ApplicationWindow) {
        let action_group = gio::SimpleActionGroup::new();
        
        // Action ouvrir fichier
        let open_file_action = gio::SimpleAction::new("open-file", None);
        let open_file_btn_clone = self.open_file_btn.clone();
        open_file_action.connect_activate(move |_, _| {
            open_file_btn_clone.activate();
        });
        action_group.add_action(&open_file_action);
        
        // Action ouvrir dossier
        let open_folder_action = gio::SimpleAction::new("open-folder", None);
        let open_folder_btn_clone = self.open_folder_btn.clone();
        open_folder_action.connect_activate(move |_, _| {
            open_folder_btn_clone.activate();
        });
        action_group.add_action(&open_folder_action);
        
        // Action convertir
        let convert_action = gio::SimpleAction::new("convert", None);
        let convert_btn_clone = self.convert_btn.clone();
        convert_action.connect_activate(move |_, _| {
            convert_btn_clone.activate();
        });
        action_group.add_action(&convert_action);
        
        window.insert_action_group("win", Some(&action_group));
        
        // Raccourcis clavier
        app.set_accels_for_action("win.open-file", &["<Ctrl>O"]);
        app.set_accels_for_action("win.open-folder", &["<Ctrl>F"]);
        app.set_accels_for_action("win.convert", &["<Ctrl>R"]);
    }
}

impl Default for HeaderBar {
    fn default() -> Self {
        Self::new()
    }
}
