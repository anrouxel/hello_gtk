use adw::{prelude::*, Application, HeaderBar, NavigationPage, NavigationView, ToolbarView, Window};
use gtk::{Box, Button, Image, Label, ListBox, ProgressBar};

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
        let toolbar_view = ToolbarView::builder().build();
        toolbar_view.add_top_bar(&headerbar);
        window.set_content(Some(&toolbar_view));
        
        // Create a navigation view
        let navigation_view = NavigationView::builder().build();
        toolbar_view.set_content(Some(&navigation_view));
        
        // Create the welcome page
        let welcome_page = Box::builder()
            .orientation(gtk::Orientation::Vertical)
            .spacing(20)
            .margin_top(20)
            .margin_bottom(20)
            .margin_start(20)
            .margin_end(20)
            .build();
        
        let cd_image = Image::from_icon_name("media-optical");
        cd_image.set_pixel_size(200);
        cd_image.set_halign(gtk::Align::Center);
        welcome_page.append(&cd_image);
        
        let welcome_title = Label::builder()
            .label("Welcome to Sound Juicer")
            .halign(gtk::Align::Center)
            .build();
        welcome_title.add_css_class("title-1");
        welcome_page.append(&welcome_title);
        
        let welcome_subtitle = Label::builder()
            .label("Select a device or open a disk image file to get started.")
            .halign(gtk::Align::Center)
            .build();
        welcome_subtitle.add_css_class("title-3");
        welcome_page.append(&welcome_subtitle);
        
        let open_button = Button::builder()
            .label("Open a Disk Image...")
            .halign(gtk::Align::Center)
            .build();
        open_button.add_css_class("suggested-action");
        open_button.add_css_class("pill");
        welcome_page.append(&open_button);
        
        // Create the tracks list page
        let tracks_page = Box::builder()
            .orientation(gtk::Orientation::Vertical)
            .spacing(20)
            .margin_top(20)
            .margin_bottom(20)
            .margin_start(20)
            .margin_end(20)
            .build();
        
        let tracks_title = Label::builder()
            .label("Red (Taylor's Version)")
            .halign(gtk::Align::Center)
            .build();
        tracks_title.add_css_class("title-1");
        tracks_page.append(&tracks_title);
        
        let artist_label = Label::builder()
            .label("Taylor Swift")
            .halign(gtk::Align::Center)
            .build();
        artist_label.add_css_class("title-3");
        tracks_page.append(&artist_label);
        
        let disc_label = Label::builder()
            .label("Disc 1 (16 tracks)")
            .halign(gtk::Align::Center)
            .build();
        disc_label.add_css_class("title-4");
        tracks_page.append(&disc_label);
        
        let rip_button = Button::builder()
            .label("Rip CD")
            .halign(gtk::Align::Center)
            .build();
        rip_button.add_css_class("suggested-action");
        rip_button.add_css_class("pill");
        tracks_page.append(&rip_button);
        
        let list_box = ListBox::builder()
            .selection_mode(gtk::SelectionMode::None)
            .build();
        
        let tracks = vec![
            "State of Grace (Taylor's Version) - 4:55",
            "Red (Taylor's Version) - 3:43",
            "Treacherous (Taylor's Version) - 4:02",
            "I Knew You Were Trouble (Taylor's Version) - 3:39",
            "All Too Well (Taylor's Version) - 5:29",
        ];
        
        for track in tracks {
            let label = Label::builder()
                .label(track)
                .halign(gtk::Align::Start)
                .build();
            list_box.append(&label);
        }
        
        tracks_page.append(&list_box);
        
        // Create the progress page
        let progress_page = Box::builder()
            .orientation(gtk::Orientation::Vertical)
            .spacing(20)
            .margin_top(20)
            .margin_bottom(20)
            .margin_start(20)
            .margin_end(20)
            .build();
        
        let progress_title = Label::builder()
            .label("Ripping CD...")
            .halign(gtk::Align::Center)
            .build();
        progress_title.add_css_class("title-1");
        progress_page.append(&progress_title);
        
        let progress_bar = ProgressBar::builder()
            .fraction(0.25)
            .halign(gtk::Align::Center)
            .build();
        progress_page.append(&progress_bar);
        
        let progress_label = Label::builder()
            .label("4/16")
            .halign(gtk::Align::Center)
            .build();
        progress_label.add_css_class("title-2");
        progress_page.append(&progress_label);
        
        // Add pages to the navigation view
        let welcome_navigation_page = NavigationPage::builder()
            .child(&welcome_page)
            .tag("welcome")
            .title("Welcome")
            .build();
        
        let tracks_navigation_page = NavigationPage::builder()
            .child(&tracks_page)
            .tag("tracks")
            .title("Tracks")
            .build();
        
        let progress_navigation_page = NavigationPage::builder()
            .child(&progress_page)
            .tag("progress")
            .title("Progress")
            .build();
        
        navigation_view.add(&welcome_navigation_page);
        navigation_view.add(&tracks_navigation_page);
        navigation_view.add(&progress_navigation_page);
        
        // Connect the open button to navigate to the tracks page
        let navigation_view_clone = navigation_view.clone();
        open_button.connect_clicked(move |_| {
            navigation_view_clone.push_by_tag("tracks");
        });
        
        // Connect the rip button to navigate to the progress page
        let navigation_view_clone2 = navigation_view.clone();
        rip_button.connect_clicked(move |_| {
            navigation_view_clone2.push_by_tag("progress");
        });
        
        window.present();
    });
    
    app.run()
}
