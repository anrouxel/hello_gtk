use adw::prelude::*;
use adw::AboutDialog;

/// Composant AboutDialog
/// Responsable de l'affichage du dialogue "À propos"
pub struct AboutDialogComponent;

impl AboutDialogComponent {
    /// Affiche le dialogue "À propos"
    pub fn show(parent: &adw::ApplicationWindow) {
        let about = AboutDialog::new();
        about.set_application_name("First App");
        about.set_version("0.1");
        about.set_comments("Application de conversion audio — exemple avec GTK-RS + Adwaita");
        about.set_developers(&["Développeur"]);
        
        about.present(Some(parent));
    }
}
