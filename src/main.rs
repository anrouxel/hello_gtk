use adw::{prelude::AdwWindowExt, Application, HeaderBar, ToolbarView, Window};
use gio::prelude::{ApplicationExt, ApplicationExtManual};
use gtk::{prelude::GtkWindowExt, Box};


fn main() -> glib::ExitCode {
    let app = Application::builder()
        .application_id("eu.anrouxel.astrid")
        .build();

    app.connect_activate(|app| {
        let window = Window::builder()
            .application(app)
            .title("Astrid")
            .default_width(800)
            .default_height(600)
            .build();

        let content = Box::builder().build();

        let headerbar = HeaderBar::builder().build();

        let toolbarview = ToolbarView::builder().build();
        toolbarview.add_top_bar(&headerbar);
        toolbarview.set_content(Some(&content));
        window.set_content(Some(&toolbarview));

        window.present();
    });

    app.run()
}
