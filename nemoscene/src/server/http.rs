use std::{
    collections::HashMap,
    error::Error,
    fmt::Display,
    io::{BufRead, BufReader, Read},
    net::TcpStream,
};

use anyhow::bail;
use log::{info, warn};

#[derive(Debug)]
pub enum HttpError {
    Redirect(String),
    NotFound(String),
    BadRequest(String),
    ServerError(anyhow::Error),
    Unspecified,
}

impl Display for HttpError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Redirect(target) => write!(f, "Redirection to {}", target),
            Self::BadRequest(msg) => write!(f, "{}", msg),
            Self::NotFound(what) => write!(f, "{}", what),
            Self::ServerError(err) => write!(f, "{}", err.to_string()),
            Self::Unspecified => write!(f, "Unspecified error"),
        }
    }
}

impl Error for HttpError {}

impl Into<i32> for &HttpError {
    fn into(self) -> i32 {
        match *self {
            HttpError::Redirect(_) => 307,
            HttpError::BadRequest(_) => 400,
            HttpError::NotFound(_) => 404,
            HttpError::ServerError(_) => 500,
            HttpError::Unspecified => 500,
        }
    }
}

#[derive(Debug, Copy, Clone)]
pub enum RequestType {
    Get,
    Post,
}

impl TryFrom<String> for RequestType {
    type Error = HttpError;
    fn try_from(value: String) -> Result<Self, Self::Error> {
        match value.to_lowercase().as_str() {
            "get" => Ok(Self::Get),
            "post" => Ok(Self::Post),
            _ => Err(HttpError::BadRequest("Invalid request method".to_string())),
        }
    }
}

#[derive(Debug, Clone)]
pub enum ParameterValue {
    String(String),
    Array(Vec<String>),
}

impl ParameterValue {
    pub fn as_string(&self) -> anyhow::Result<&String> {
        if let Self::String(s) = self {
            Ok(s)
        } else {
            bail!("Invalid parameter value")
        }
    }

    pub fn as_array(&self) -> anyhow::Result<&Vec<String>> {
        if let Self::Array(a) = self {
            Ok(a)
        } else {
            bail!("Invalid parameter value")
        }
    }
}

impl PartialEq for ParameterValue {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::String(this), Self::String(that)) => this == that,
            (Self::Array(this), Self::Array(that)) => this == that,
            _ => false,
        }
    }
}

#[derive(Debug, Clone)]
pub struct HttpRequest {
    pub method: RequestType,
    pub route: String,
    pub header: HashMap<String, String>,
    pub get: Option<HashMap<String, ParameterValue>>,
    pub post: Option<HashMap<String, ParameterValue>>,
}

impl HttpRequest {
    pub fn read_from_stream(stream: &TcpStream) -> anyhow::Result<HttpRequest> {
        let mut reader = BufReader::new(stream);
        let mut first_line = String::new();
        if let Err(err) = reader.read_line(&mut first_line) {
            bail!(err)
        }

        let mut header: HashMap<String, String> = HashMap::new();
        loop {
            let mut line = String::new();
            if let Err(err) = reader.read_line(&mut line) {
                bail!(err)
            }
            if line == "\r\n" {
                break;
            }
            if let Some(semicolon) = line.find(':') {
                header.insert(
                    line[..semicolon].to_string(),
                    line[semicolon + 2..line.len() - 2].to_string(),
                );
            }
        }

        let first_line: Vec<String> = first_line.split(" ").map(|s| s.to_string()).collect();

        let method = if let Some(method) = first_line.get(0) {
            if let Ok(method) = RequestType::try_from(method.clone()) {
                method
            } else {
                bail!(HttpError::BadRequest("Corrupted HTTP headers".to_string()))
            }
        } else {
            bail!(HttpError::BadRequest("Corrupted HTTP headers".to_string()))
        };
        let mut route = if let Some(route) = first_line.get(1) {
            route.clone()
        } else {
            bail!(HttpError::BadRequest("Corrupted HTTP headers".to_string()))
        };

        let get = if let Some(question_mark) = route.find('?') {
            let get = Self::parse_parameters(&route[question_mark + 1..])?;
            route = route[..question_mark].to_string();
            Some(get)
        } else {
            None
        };

        let post = if let (Some(content_length), Some(content_type)) = (
            if let Some(content_length) = header.get("Content-Length") {
                content_length
                    .parse::<usize>()
                    .map_or(None, |len| Some(len))
            } else {
                None
            },
            header.get("Content-Type"),
        ) {
            if content_type == "application/x-www-form-urlencoded" && content_length > 0 {
                let mut buffer: Vec<u8> = vec![0; content_length];
                reader.read(buffer.as_mut_slice())?;
                Some(Self::parse_parameters(String::from_utf8(buffer)?.as_str())?)
            } else {
                None
            }
        } else {
            None
        };

        Ok(HttpRequest {
            method,
            route,
            header,
            get,
            post,
        })
    }

    fn parse_parameters(parameters: &str) -> anyhow::Result<HashMap<String, ParameterValue>> {
        let mut params: HashMap<String, ParameterValue> = HashMap::new();
        for param in parameters.split('&') {
            if let Some(equal_sign) = param.find('=') {
                let key = url_escape::decode(&param[..equal_sign]).to_string();
                let value = url_escape::decode(&param[equal_sign + 1..]).to_string();
                if key.ends_with("[]") {
                    let key = key[..key.len() - 2].to_string();
                    let par = params.get_mut(&key);
                    if let None = par {
                        params.insert(key, ParameterValue::Array(vec![value]));
                    } else if let Some(ParameterValue::Array(par)) = par {
                        par.push(value);
                    } else {
                        bail!(HttpError::BadRequest(String::from("Invalid request body")))
                    }
                } else {
                    params.insert(key, ParameterValue::String(value));
                }
            }
        }
        Ok(params)
    }
}
