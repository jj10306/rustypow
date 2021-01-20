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

    pub fn get_url(&self) -> &str {
        &self.url
    }

    pub fn get_login_email(&self) -> &str {
        &self.login_email
    }

    pub fn get_login_password(&self) -> &str {
        &self.login_password
    }

    pub fn get_notify_username(&self) -> &str {
        &self.notify_username
    }

    pub fn get_notify_password(&self) -> &str {
        &self.notify_password
    }
}

pub struct ReservationInfo {
    // {
    //      Location:
    //          {
    //              Date: [Email]
    //          }
    // }
    inner: HashMap<String, HashMap<String, Vec<String>>>
}

impl ReservationInfo {
    pub fn new(path: &str) -> ReservationInfo {
        let mut inner: HashMap<String, HashMap<String, Vec<String>>> = HashMap::new();
        let file = File::open(path).expect("Error openenign file");
        let reader = BufReader::new(file);

        // Read the JSON contents of the file as an instance of `User`.
        let v: Value = serde_json::from_reader(reader).expect("Error when parsing json");
        match(v) {
            Value::Object(email_map) => {
                for email in email_map.keys() {
                    let object = email_map.get(email).unwrap();
                    match(object) {
                        Value::Object(location_map) => {
                            for location in location_map.keys() {
                                let dates = location_map.get(location).unwrap().as_array().expect("Should be an array");
                                for date in dates.iter() {
                                    let date = date.as_str().unwrap().to_string();
                                    if (inner.contains_key(location)){
                                        let day_map = inner.get_mut(location).unwrap();
                                        if (day_map.contains_key(&date)) {
                                            day_map.get_mut(&date).unwrap().push(email.clone());
                                        } else {
                                            day_map.insert(date, vec![email.clone()]);
                                        }
                                    } else {
                                        let mut intermediate = HashMap::new();
                                        intermediate.insert(date, vec![email.clone()]);
                                        inner.insert(location.clone(), intermediate);
                                    }
                                }
                            } 
                        },
                        _ => panic!("should have been an object")
                    }
                }
            },
            _ => panic!("should have been an object")
        }
        ReservationInfo { inner } 
    }
    pub fn get_locations(&self) -> Vec<&String> {
        self.inner.keys().collect()
    }
    pub fn get_dates(&self, location: &str) -> Vec<&String> {
        // TODO: change the return type to a set
        self.inner.get(location).unwrap().keys().collect()
    }

    pub fn get_emails(&self, location: &str, date: &str) -> &Vec<String> {
        self.inner.get(location).unwrap().get(date).unwrap()
    }
}


