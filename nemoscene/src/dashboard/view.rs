use html_to_string_macro::html;
use crate::dashboard::Point;

#[derive(Debug, Clone)]
pub struct View {
    pub uuid: String,
    pub position: Point,
    pub size: Point,
}

impl View {
    pub fn iframe(&self) -> String {
        html!(
            <iframe
                width=self.size.x
                height=self.size.y
                style=format!("left: {}px; top: {}px;", self.position.x, self.position.y)
                src=format!("http://localhost:1337/bundle/{}", self.uuid)>
            </iframe>
        )
    }
}