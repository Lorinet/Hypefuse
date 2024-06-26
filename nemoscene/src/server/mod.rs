pub mod http;
pub mod threadpool;

use html_to_string_macro::*;
use log::{error, info, warn};
use once_cell::sync::Lazy;
use std::{
    collections::HashMap,
    error::Error,
    fs,
    io::{prelude::*, BufReader},
    net::{TcpListener, TcpStream},
    sync::*,
    thread,
    time::Duration,
};
use std::collections::BTreeMap;
use std::fs::File;
use std::path::PathBuf;
use anyhow::anyhow;
use crate::get_system_state;

use self::http::{HttpError, ParameterValue};

static CONTENT_TYPES: Lazy<BTreeMap<&str, &str>> = Lazy::new(|| BTreeMap::from([
    ("html", "text/html"),
    ("txt", "text/plain"),
    ("ttf", "font/ttf"),
    ("ico", "image/x-icon"),
    ("js", "text/javascript"),
    ("json", "application/json"),
]));

pub fn run_server() -> ! {
    let listener = TcpListener::bind("0.0.0.0:1337").unwrap();
    let pool = threadpool::ThreadPool::new(4);
    info!("Accepting clients");
    loop {
        let (stream, address) = match listener.accept() {
            Err(err) => {
                error!("Client connection error: {}", err);
                continue;
            }
            Ok((stream, address)) => (stream, address),
        };
        pool.execute(move || {
            if let Err(err) = handle_connection(stream) {
                error!("Server error: {:?}", err);
            }
        });
    }
}

fn handle_connection(
    mut stream: TcpStream,
) -> anyhow::Result<()> {
    let request = http::HttpRequest::read_from_stream(&stream)?;
    //info!("Read request: {:#?}", request);
    let mut parts = request.route.split('/');
    parts.next();
    let result = if let Ok(request_type) = parts.next().ok_or(anyhow!("Invalid request")) {
        if request_type == "bundle" {
            if let Some(uuid) = parts.next() {
                let route = parts.map(|s| s.to_string()).collect::<Vec<String>>().join("/");
                match serve_file(
                    uuid,
                    route.as_str(),
                    request.get,
                    request.post,
                ) {
                    Err(error) => Err(error),
                    Ok((content_type, content)) => respond(&mut stream, 200, content_type, content)
                }
            } else {
                Err(anyhow!("Invalid request"))
            }
        } else if request_type == "config" {
            match serve_config(request.get) {
                Err(error) => Err(error),
                Ok(content) => respond(&mut stream, 200, String::from(*CONTENT_TYPES.get("json").unwrap()), content),
            }
        } else if request_type == "favicon.ico" {
            respond(&mut stream, 200, String::from(*CONTENT_TYPES.get("ico").unwrap()), Vec::new())
        } else {
            Err(anyhow!("Invalid request type: {}", request_type))
        }
    } else {
        Err(anyhow!("Invalid request"))
    };
    if let Err(error) = result {
        error_response(&mut stream, error.to_string(), Some(error.backtrace().to_string()))
    } else {
        Ok(())
    }
}

fn serve_config(get: Option<HashMap<String, ParameterValue>>) -> anyhow::Result<Vec<u8>> {
    if let Some(get) = get {
        if let (Some(uuid), Some(base), Some(key)) = (get.get("uuid").map(|v| v.as_string().cloned().unwrap_or(String::new())), get.get("base").map(|v| v.as_string().cloned().unwrap_or(String::new())), get.get("key").map(|v| v.as_string().cloned().unwrap_or(String::new()))) {
            let path = format!("data/bundles/{}/config/{}", uuid, base);
            if let Some(base) = get_system_state!().configuration.get_base(path.as_str()) {
                base.get_json(key.as_str()).ok_or(anyhow!("Invalid configuration key: {}", key))
            } else {
                Err(anyhow!("Invalid configuration base: {}", base))
            }
        } else {
            Err(anyhow!("Invalid request"))
        }
    } else {
        Err(anyhow!("Invalid request"))
    }
}

fn serve_file(
    uuid: &str,
    route: &str,
    get: Option<HashMap<String, ParameterValue>>,
    post: Option<HashMap<String, ParameterValue>>,
) -> anyhow::Result<(String, Vec<u8>)> {
    let base_path = {
        get_system_state!().app_manager.get_bundle(uuid).ok_or(anyhow!("Bundle not found: {}", uuid))?.base_path.clone()
    };
    let mut route = PathBuf::from(route);
    if route.components().count() == 0 {
        route = PathBuf::from("app/index.html");
    }
    let path = PathBuf::from(base_path).join(route);
    let extension = String::from(path.extension().map_or("txt", |e| e.to_str().unwrap()));
    let path = path.to_str().unwrap().to_string();
    let content = fs::read(path)?;
    let content_type = String::from(*CONTENT_TYPES.get(extension.as_str()).unwrap_or(&"txt"));
    Ok((content_type, content))
}

fn error_response(stream: &mut TcpStream, message: String, backtrace: Option<String>) -> anyhow::Result<()> {
    respond(stream, 500, String::from(*CONTENT_TYPES.get("txt").unwrap()), format!("{}\n{}", message, backtrace.or(Some(String::new())).unwrap()).into_bytes())
}

fn respond(stream: &mut TcpStream, status_code: i32, content_type: String, contents: Vec<u8>) -> anyhow::Result<()> {
    let response = if status_code == 307 {
        format!(
            "{}\r\nLocation: {}\r\n\r\n",
            "HTTP/1.1 307 OK",
            unsafe { std::str::from_utf8_unchecked(contents.as_slice()) },
        )
    } else {
        format!(
            "{}\r\nContent-Length: {}\r\nContent-Type: {}\r\n\r\n{}",
            format!("HTTP/1.1 {} OK", status_code),
            contents.len(),
            content_type,
            unsafe { std::str::from_utf8_unchecked(contents.as_slice()) },
        )
    };
    stream.write_all(response.as_bytes())?;
    stream.flush()?;
    Ok(())
}

#[macro_export]
macro_rules! redirect {
    ($target:expr) => {
        {
            use anyhow::bail;
            use crate::server::http::HttpError;
            bail!(HttpError::Redirect($target.to_string()))
        }
    }
}

#[macro_export]
macro_rules! database_conn {
    ($database:expr) => {
        &mut $database.clone().get().unwrap()
    }
}