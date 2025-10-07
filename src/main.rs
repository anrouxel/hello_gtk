use adw::{prelude::AdwWindowExt, Application, HeaderBar, ToolbarView, Window};
use gio::prelude::{ApplicationExt, ApplicationExtManual};
use gtk::prelude::GtkWindowExt;


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


        let headerbar = HeaderBar::builder().build();

        let toolbarview = ToolbarView::builder().build();
        toolbarview.set_title_widget(Some(&headerbar));
        window.set_content(Some(&toolbarview));

        window.present();
    });

    app.run()
}
