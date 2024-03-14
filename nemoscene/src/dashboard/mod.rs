use core::ffi::c_ssize_t;
use std::collections::BTreeMap;
use std::ops::{Add, Mul};
use std::sync::{Arc, mpsc, Mutex};
use std::sync::mpsc::channel;
use std::thread;
use std::time::Duration;
use anyhow::anyhow;
use gtk::{CssProvider, Fixed, glib, Window, WindowType};
use gtk::ffi::{gtk_css_provider_get_default, gtk_css_provider_load_from_data, gtk_style_context_add_provider_for_screen, GtkStyleProvider};
use gtk::gdk::ffi::gdk_screen_get_default;
use gtk::glib::ffi::GError;
use gtk::prelude::{ContainerExt, CssProviderExt, GtkWindowExt, WidgetExt};
use log::{error, info};
use view::View;
use crate::configuration::{ConfigurationBase, ConfigurationRegistry};
use crate::dashboard::view::ViewParameters;
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

#[derive(Debug)]
pub enum DashboardMessage {
    Quit,
    AttachView(ViewParameters),
}

#[derive(Copy, Clone, Debug)]
pub struct Viewport {
    pub screen_size: Point,
    pub pixel_ratio: Point,
}

impl Default for Viewport {
    fn default() -> Self {
        Viewport {
            screen_size: Point::default(),
            pixel_ratio: Point::default(),
        }
    }
}

impl Viewport {
    fn to_actual_pixels(&self, point: Point) -> Point {
        point * self.pixel_ratio
    }
}

pub struct Dashboard {
    viewport: Viewport,
    channel_sender: Option<glib::Sender<DashboardMessage>>,
}

impl Dashboard {
    pub fn new() -> Dashboard {
        Dashboard {
            viewport: Viewport::default(),
            channel_sender: None,
        }
    }

    pub fn init(&mut self, config: &ConfigurationRegistry) -> anyhow::Result<()> {
        let screen_width: i32 = {
            config.get_base("data/configuration/dashboard").ok_or(anyhow!("Cannot load configuration base"))?.get_i64("screen_width").ok_or(anyhow!("Cannot load screen size from configuration"))? as i32
        };
        let screen_height: i32 = {
            config.get_base("data/configuration/dashboard").ok_or(anyhow!("Cannot load configuration base"))?.get_i64("screen_height").ok_or(anyhow!("Cannot load screen size from configuration"))? as i32
        };

        let (sender_sender, sender_receiver) = mpsc::channel();

        let viewport = Viewport {
            screen_size: Point::new_i32(screen_width, screen_height),
            pixel_ratio: Point::new_f32(screen_width as f32 / 1000.00, screen_height as f32 / 1000.00),
        };

        thread::spawn(move || Self::ui_thread(viewport.clone(), sender_sender));

        self.channel_sender = Some(sender_receiver.recv().expect("Sender thread sender receiver sender channel broken"));

        for widget in config.get_bases_of("data/configuration/widgets") {
            if let Ok(widget) = Self::load_widget(widget) {
                if let Err(error) = self.send_message(DashboardMessage::AttachView(widget.clone())) {
                    error!("Failed to attach widget {:#?}: {}", widget, error);
                }
            }
        }


        Ok(())
    }

    fn load_widget(base: &ConfigurationBase) -> anyhow::Result<ViewParameters> {
        let uuid = base.get_str("uuid").ok_or(anyhow!("Invalid widget configuration"))?;
        let position_x = base.get_i64("position_x").ok_or(anyhow!("Invalid widget configuration"))? as i32;
        let position_y = base.get_i64("position_y").ok_or(anyhow!("Invalid widget configuration"))? as i32;
        let width = base.get_i64("width").ok_or(anyhow!("Invalid widget configuration"))? as i32;
        let height = base.get_i64("height").ok_or(anyhow!("Invalid widget configuration"))? as i32;
        Ok(ViewParameters {
            uuid,
            url: None,
            position: Point::new_i32(position_x, position_y),
            size: Point::new_i32(width, height),
        })
    }

    pub fn send_message(&self, message: DashboardMessage) -> anyhow::Result<()> {
        self.channel_sender.as_ref().expect("Dashboard message channel not initialized yet").send(message)?;
        Ok(())
    }

    fn ui_thread(viewport: Viewport, sender_sender: mpsc::Sender<glib::Sender<DashboardMessage>>) {
        gtk::init().unwrap();
        unsafe { Self::load_css() };
        let window = Window::new(WindowType::Toplevel);
        window.set_decorated(false);
        window.set_size_request(viewport.screen_size.x_i32(), viewport.screen_size.y_i32());
        window.show_all();

        let container = Fixed::new();
        window.set_child(Some(&container));

        let (sender, receiver) = glib::MainContext::channel(glib::Priority::DEFAULT);

        let mut views: BTreeMap<String, View> = BTreeMap::new();

        receiver.attach(None, move |message| {
            //info!("Message: {:?}", message);
            match message {
                DashboardMessage::Quit => {
                    window.close();
                    SystemState::shutdown();
                }
                DashboardMessage::AttachView(view) => Self::attach_view(&window, &container, &viewport, &mut views, view),
            };
            glib::ControlFlow::Continue
        });

        sender_sender.send(sender).unwrap();

        gtk::main();
    }

    fn attach_view(window: &Window, container: &Fixed, viewport: &Viewport, views: &mut BTreeMap<String, View>, view: ViewParameters) {
        let uuid = view.uuid.clone();
        let mut view = View::new(view);
        view.parameters.position = viewport.to_actual_pixels(view.parameters.position);
        view.parameters.size = viewport.to_actual_pixels(view.parameters.size);
        views.insert(uuid.clone(), view);
        views.get(&uuid).unwrap().attach_view(container);
        window.show_all();
    }

    unsafe fn load_css() {
        let provider = CssProvider::new();
        provider.load_from_data(include_bytes!("dashboard_style_gtk.css")).expect("Cannot load GTK style data");

        let css_file = include_bytes!("dashboard_style_gtk.css");
        let provider = gtk_css_provider_get_default();
        gtk_css_provider_load_from_data(provider, css_file.as_ptr(), css_file.len() as c_ssize_t, 0 as *mut *mut GError);
        gtk_style_context_add_provider_for_screen(
            gdk_screen_get_default(),
            provider as *mut GtkStyleProvider,
            gtk::STYLE_PROVIDER_PRIORITY_APPLICATION,
        );
    }
}
