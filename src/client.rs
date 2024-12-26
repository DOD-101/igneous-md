use gtk::{prelude::*, Window, WindowType};
use std::process::exit;
use webkit2gtk::{CacheModel, WebContext, WebContextExt, WebView, WebViewExt};

#[derive(Debug)]
pub struct Viewer {
    addr: String,
}

impl Viewer {
    pub fn new(addr: String) -> Self {
        Viewer { addr }
    }

    pub fn start(&self) {
        log::info!("Starting client on {}", self.addr);
        if gtk::init().is_err() {
            log::error!("Failed to init gtk. Needed for viewer.");
            exit(1)
        }

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

        gtk::main()
    }
}
