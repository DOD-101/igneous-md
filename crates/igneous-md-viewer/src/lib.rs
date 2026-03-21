use std::fmt::Display;

use gtk4::{gio, glib, prelude::*, Application, ApplicationWindow};
use webkit6::{prelude::*, CacheModel, Settings, URISchemeRequest, WebContext, WebView};

/// A struct representing the igneous-md markdown viewer.
#[derive(Debug)]
pub struct Viewer<'a> {
    addr: Address<'a>,
}

const APP_ID: &str = "dod.igneous-md.viewer";

impl<'a> Viewer<'a> {
    /// Create a new [Viewer]
    pub fn new(addr: Address<'a>) -> Self {
        Viewer { addr }
    }

    /// Start the viewer
    pub fn start(&self) {
        let app = Application::builder().application_id(APP_ID).build();

        let addr = self.addr.to_string();
        app.connect_activate(move |app| {
            Self::build_ui(&addr, app);
        });

        app.run_with_args::<&str>(&[]);
    }

    /// Build the actual GTK UI
    fn build_ui(addr: &str, app: &Application) {
        let window = ApplicationWindow::builder()
            .application(app)
            .title("igneous-md viewer")
            .build();

        let context = WebContext::default().unwrap();
        context.set_cache_model(CacheModel::DocumentBrowser);
        // TODO: Not sure this will behave properly with relative paths in all cases. Needs further
        // testing.
        //
        // TODO: Document this as necessary for writing your own viewer
        context.register_uri_scheme("asset", |req: &URISchemeRequest| {
            let uri = req.uri().unwrap();
            let path = uri.strip_prefix("asset://").unwrap();

            match std::fs::read(path) {
                Ok(bytes) => {
                    let mime = mime_guess::from_path(path)
                        .first_or_octet_stream()
                        .to_string();
                    let stream = gio::MemoryInputStream::from_bytes(&glib::Bytes::from(&bytes));
                    req.finish(&stream, bytes.len() as i64, Some(&mime));
                }
                Err(e) => {
                    req.finish_error(&mut glib::Error::new(
                        gio::IOErrorEnum::NotFound,
                        &e.to_string(),
                    ));
                }
            }
        });

        let view = WebView::builder()
            .web_context(&context)
            .settings(&Settings::builder().enable_developer_extras(true).build())
            .build();

        view.show();

        window.set_child(Some(&view));
        window.present();
        // view.load_uri(&format!("http://{}", addr));
        view.load_html(
            include_str!(concat!(env!("OUT_DIR"), "/index.html")),
            Some("viewer"),
        );
    }
}

#[derive(Debug, Default, Clone, Copy)]
pub struct Address<'a> {
    host: &'a str,
    port: u16,
    update_rate: u64,
    css: Option<&'a str>,
}

impl<'a> Address<'a> {
    pub fn new(host: &'a str, port: u16, update_rate: u64, css: Option<&'a str>) -> Self {
        Self {
            host,
            port,
            update_rate,
            css,
        }
    }
}

impl Display for Address<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}:{}/?update_rate={}{}",
            self.host,
            self.port,
            self.update_rate,
            self.css.map(|s| format!("&css={}", s)).unwrap_or_default(),
        )
    }
}
