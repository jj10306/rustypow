use std::vec::Vec;
use std::string::String;
use std::boxed::Box;
use std::collections::{HashSet, HashMap};
use std::{thread, time};
use std::cell::RefCell;
use std::error::Error;
use std::fs::File;
use std::io::BufReader;
use std::path::Path;
use thirtyfour_sync::prelude::*;
use serde::Deserialize;
use serde_json::Value;
use lettre::transport::smtp::authentication::Credentials;
use lettre::{Message, SmtpTransport, Transport};
use log;
use percent_encoding::percent_decode;
use reqwest::header::HeaderName;
use chrono::{Datelike, Timelike, Utc};

#[derive(Deserialize)]
pub struct Config {
    url: String,
    login_email: String,
    login_password: String,
    notify_username: String,
    notify_password: String 
}
impl Config {
    pub fn new(path: &str) -> Config {
        // Open the file in read-only mode with buffer.
        let file = File::open(path).expect("Error openenign file");
        let reader = BufReader::new(file);
        // Read the JSON contents of the file as an instance of `User`.
        let u = serde_json::from_reader(reader).expect("Error when parsing json");
        u
    }

    fn get_url(&self) -> &str {
        &self.url
    }

    fn get_login_email(&self) -> &str {
        &self.login_email
    }

    fn get_login_password(&self) -> &str {
        &self.login_password
    }

    fn get_notify_username(&self) -> &str {
        &self.notify_username
    }

    fn get_notify_password(&self) -> &str {
        &self.notify_password
    }
}
