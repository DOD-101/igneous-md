use gtk::{prelude::*, Window, WindowType};
use std::{io, path::PathBuf, process::exit, time::SystemTime};
use uuid::Uuid;
use webkit2gtk::{CacheModel, WebContext, WebContextExt, WebView, WebViewExt};

use crate::{config::Config, convert::md_to_html, paths::Paths};

#[derive(Debug)]
pub struct Client {
    #[allow(dead_code)]
    id: uuid::Uuid,
    md_path: PathBuf,
    last_modified: SystemTime,
    md: String,
    html: String,
    pub config: Config,
}

pub enum MdChanged {
    Changed(SystemTime),
    NotChanged,
}

impl Client {
    pub fn new(md_path: PathBuf, paths: Paths) -> io::Result<Self> {
        Ok(Self {
            id: Uuid::new_v4(),
            md_path,
            config: Config::new(paths)?,
            html: String::new(),
            md: String::new(),
            last_modified: SystemTime::UNIX_EPOCH,
        })
    }

    fn update_md(&mut self) -> io::Result<()> {
        self.md = std::fs::read_to_string(&self.md_path)?;

        Ok(())
    }

    pub fn changed(&self) -> io::Result<MdChanged> {
        let last_modified = std::fs::metadata(&self.md_path)?.modified()?;

        if last_modified != self.last_modified {
            Ok(MdChanged::Changed(last_modified))
        } else {
            Ok(MdChanged::NotChanged)
        }
    }

    #[allow(dead_code)]
    pub fn get_md_path(&self) -> PathBuf {
        self.md_path.clone()
    }

    pub fn get_html(&self) -> String {
        self.html.clone()
    }

    #[allow(dead_code)]
    pub fn get_latest_html(&mut self) -> io::Result<String> {
        Ok(self
            .get_latest_html_if_changed()?
            .unwrap_or(self.html.clone()))
    }

    pub fn get_latest_html_if_changed(&mut self) -> io::Result<Option<String>> {
        if let MdChanged::Changed(time) = self.changed()? {
            self.last_modified = time;
        } else {
            return Ok(None);
        }

        self.update_md()?;

        self.html = md_to_html(&self.md);

        Ok(Some(self.html.clone()))
    }
}

// TODO: This viewer isn't create here and I don't like having to call new just to then call start
// right after. It might be best not to have a struct for this.
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
