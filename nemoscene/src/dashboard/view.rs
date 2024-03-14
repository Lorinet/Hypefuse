use gtk::Fixed;
use webkit2gtk::{WebContext, WebView};
use crate::*;
use crate::dashboard::Point;

#[derive(Debug, Clone)]
pub struct ViewParameters {
    pub uuid: String,
    pub url: Option<String>,
    pub position: Point,
    pub size: Point,
}

pub struct View {
    pub parameters: ViewParameters,
    web_context: WebContext,
    web_view: WebView,
}

impl View {
    pub fn new(parameters: ViewParameters) -> View {
        let web_context = WebContext::default().unwrap();
        let web_view = WebView::with_context(&web_context);
        if let Some(url) = &parameters.url {
            web_view.load_uri(url.as_str());
        } else {
            web_view.load_uri(format!("http://localhost:1337/bundle/{}", &parameters.uuid).as_str());
        }
        View {
            web_context,
            web_view,
            parameters,
        }
    }

    pub fn attach_view(&self, fixed: &Fixed) {
        self.web_view.set_size_request(self.parameters.size.x_i32(), self.parameters.size.y_i32());
        fixed.put(&self.web_view, self.parameters.position.x_i32(), self.parameters.position.y_i32());
    }
}

impl<'a> View {
    pub fn web_view(&'a self) -> &'a WebView {
        &self.web_view
    }
}