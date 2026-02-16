use gtk4::{prelude::*, Application, ApplicationWindow};
use webkit6::{prelude::*, CacheModel, WebContext, WebView};

/// A struct representing the igneous-md markdown viewer.
#[derive(Debug)]
pub struct Viewer {
    addr: String,
}

const APP_ID: &str = "dod.igneous-md.viewer";

impl Viewer {
    /// Create a new [Viewer]
    pub fn new(addr: String) -> Self {
        Viewer { addr }
    }

    /// Start the viewer
    pub fn start(&self) {
        let app = Application::builder().application_id(APP_ID).build();

        let addr = self.addr.clone();
        app.connect_activate(move |app| {
            Self::build_ui(addr.clone(), app);
        });

        app.run_with_args::<&str>(&[]);
    }

    /// Build the actual GTK UI
    fn build_ui(addr: String, app: &Application) {
        let window = ApplicationWindow::builder()
            .application(app)
            .title("igneous-md viewer")
            .build();

        let context = WebContext::default().unwrap();
        context.set_cache_model(CacheModel::DocumentBrowser);

        let view = WebView::builder().web_context(&context).build();

        view.show();

        window.set_child(Some(&view));
        window.present();
        view.load_uri(&format!("http://{}", addr));
    }
}
