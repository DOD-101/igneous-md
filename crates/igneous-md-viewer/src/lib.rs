use gtk::{glib, prelude::*, Window, WindowType};
use webkit2gtk::{CacheModel, WebContext, WebContextExt, WebView, WebViewExt};

/// A struct representing the igneous-md markdown viewer.
#[derive(Debug)]
pub struct Viewer {
    addr: String,
}

impl Viewer {
    /// Create a new [Viewer]
    pub fn new(addr: String) -> Self {
        Viewer { addr }
    }

    /// Start the viewer, exiting the program if it fail's.
    pub fn start(&self) -> Result<(), glib::BoolError> {
        gtk::init()?;

        let window = Window::new(WindowType::Toplevel);
        window.set_title("igneous-md viewer");
        window.set_default_size(800, 600);

        let context = WebContext::default().unwrap();
        context.set_cache_model(CacheModel::DocumentViewer);
        context.clear_cache();

        let view = WebView::with_context(&context);

        view.load_uri(&format!("http://{}", self.addr));

        window.add(&view);

        window.show_all();

        gtk::main();

        Ok(())
    }
}
