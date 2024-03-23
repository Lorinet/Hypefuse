use std::collections::BTreeMap;
use std::ops::{Add, Mul};
use std::sync::{Arc, mpsc, Mutex};
use std::sync::mpsc::channel;
use std::thread;
use std::time::Duration;
use anyhow::anyhow;
use html_to_string_macro::html;
use log::{error, info};
use view::View;
use crate::configuration::{ConfigurationBase, ConfigurationRegistry};
use crate::system_state::SystemState;

pub mod view;

#[derive(Copy, Clone, Debug)]
pub struct Point {
    x: f32,
    y: f32,
}

impl Default for Point {
    fn default() -> Self {
        Point {
            x: 0.00,
            y: 0.00,
        }
    }
}

impl Mul for Point {
    type Output = Self;

    fn mul(self, rhs: Self) -> Self::Output {
        Point {
            x: self.x * rhs.x,
            y: self.y * rhs.y,
        }
    }
}

impl Add for Point {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Point {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
        }
    }
}

impl Point {
    pub fn x_i32(&self) -> i32 {
        self.x as i32
    }

    pub fn y_i32(&self) -> i32 {
        self.y as i32
    }

    pub fn x_f32(&self) -> f32 {
        self.x
    }

    pub fn y_f32(&self) -> f32 {
        self.y
    }

    pub fn new_i32(x: i32, y: i32) -> Point {
        Point {
            x: x as f32,
            y: y as f32,
        }
    }

    pub fn new_f32(x: f32, y: f32) -> Point {
        Point {
            x,
            y,
        }
    }
}

pub struct Dashboard {
    widgets: Vec<View>,
    reload_requested: bool,
}

impl Dashboard {
    pub fn new() -> Dashboard {
        Dashboard {
            widgets: Vec::new(),
            reload_requested: true,
        }
    }

    pub fn init(&mut self, config: &ConfigurationRegistry) -> anyhow::Result<()> {
        info!("Initializing Dashboard...");
        self.widgets.clear();
        for widget in config.get_bases_of_bundle("widgets") {
            info!("Loading widget for '{}'", widget.get_str("uuid").unwrap_or("undefined".to_string()));
            match Self::load_widget(widget) {
                Ok(widget) => self.widgets.push(widget),
                Err(error) => error!("Failed to attach widget {:#?}: {}", widget, error),
            }
        }
        self.reload_requested = true;
        Ok(())
    }

    fn load_widget(base: &ConfigurationBase) -> anyhow::Result<View> {
        let uuid = base.get_str("uuid").ok_or(anyhow!("Invalid widget configuration"))?;
        let position_x = base.get_i64("position_x").ok_or(anyhow!("Invalid widget configuration"))? as i32;
        let position_y = base.get_i64("position_y").ok_or(anyhow!("Invalid widget configuration"))? as i32;
        let width = base.get_i64("width").ok_or(anyhow!("Invalid widget configuration"))? as i32;
        let height = base.get_i64("height").ok_or(anyhow!("Invalid widget configuration"))? as i32;
        Ok(View {
            uuid,
            position: Point::new_i32(position_x, position_y),
            size: Point::new_i32(width, height),
        })
    }

    pub fn set_reload_requested(&mut self, reload_requested: bool) {
        self.reload_requested = reload_requested;
    }

    pub fn get_reload_requested(&self) -> bool {
        self.reload_requested
    }

    pub fn serve(&mut self) -> String {
        self.set_reload_requested(false);
        html!(
            "<!DOCTYPE html>"
            <html>
                <head>
                    <title>"Hypefuse Dashboard"</title>
                <style>
                { include_str!("dashboard_style.css") }
                </style>
                <script>
                { include_str!("dashboard_script.js") }
                </script>
                </head>
                <body>
                    { self.widgets.iter().map(|w| w.iframe()).collect::<Vec<String>>().join("\n").to_string() }
                </body>
            </html>
        )
    }
}
