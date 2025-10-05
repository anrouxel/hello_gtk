mod components;
mod models;
mod services;

use crate::components::main_window::MainWindow;
use adw::prelude::*;
use gtk::glib;

fn build_ui(app: &adw::Application) {
    let window = MainWindow::new(app);
    window.present();
}

fn main() -> glib::ExitCode {
    let app = adw::Application::builder()
        .application_id("com.example.FirstAdwaitaApp")
        .build();
    
    app.connect_activate(build_ui);
    
    app.run()
}
