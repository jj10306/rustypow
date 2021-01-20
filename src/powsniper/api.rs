use crate::utils::config::Config;
use crate::utils::reservation_info::ReservationInfo;

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


pub struct ApiPowSniper {
    client: reqwest::blocking::Client,
    config: Config,
    reservations: ReservationInfo,
    current_available: RefCell<HashSet<String>>
}
impl ApiPowSniper {
    pub fn new(config: Config, reservations: ReservationInfo) -> Result<ApiPowSniper, Box<dyn std::error::Error>> {
        let client = reqwest::blocking::Client::builder().cookie_store(true).build()?;
        let current_available = RefCell::new(HashSet::new());
        Ok(ApiPowSniper { client, config, reservations, current_available })
    }

    pub fn run(&self) -> Result<(), Box<dyn std::error::Error>> {
        let wait = time::Duration::from_secs(30);
        loop {
            let locations = self.reservations.get_locations();
            let ping_token = self.ping()?;
            let session_token = self.session(ping_token)?;
            let resort_ids = self.resort_request()?;
            log::debug!("{:?}", self.current_available);
            for location in locations {
                log::debug!("Running... Checking {}", location);
                let unavailable_dates = self.get_unavailable_dates(&resort_ids, location)?;
                let dates = self.reservations.get_dates(location);
                for date in dates {
                    let hash = location.to_owned() + date;
                    if unavailable_dates.contains(date) {
                        self.current_available.borrow_mut().remove(&hash); 
                    } else {
                        if self.current_available.borrow_mut().insert(hash) {
                            let emails = self.reservations.get_emails(location, date);
                            for email in emails.iter() {
                                self.notify(email, location, date);
                            }
                            self.notify("jjohnson473@gatech.edu", location, date);
                        }
                    }
                }
            }
            thread::sleep(wait);
        }
        Ok(())
    }

    pub fn ping(&self) -> Result<String, Box<dyn std::error::Error>> { 
        let ping_url = "https://account.ikonpass.com/api/v2/ping";
        let ping_resp = self.client.get(ping_url).send()?;
        for cookie in ping_resp.cookies() {
            if cookie.name() == "PROD-XSRF-TOKEN" {
                let raw_token = cookie.value().to_string();
                let decoded_token = percent_decode(raw_token.as_bytes()).decode_utf8().unwrap().into_owned(); 
                return Ok(decoded_token);
            }
        }
        Err("No token found".into())
    }
    fn session(&self, token: String) -> Result<String, Box<dyn std::error::Error>> { 
        let session_url = "https://account.ikonpass.com/session";
        let mut map = HashMap::new();
        map.insert("email", self.config.get_login_email());
        map.insert("password", self.config.get_login_password());
        let session_resp = self.client.put(session_url).header("x-csrf-token", token).json(&map).send()?;
        for cookie in session_resp.cookies() {
            if cookie.name() == "_itw_iaa_prod_session" {
                let raw_token = cookie.value().to_string();
                let decoded_token = percent_decode(raw_token.as_bytes()).decode_utf8().unwrap().into_owned(); 
                return Ok(decoded_token);
            }
        }
        Err("No token found".into())
    }

    fn resort_request(&self) -> Result<HashMap<String, u64>, Box<dyn std::error::Error>>  {
        let url = "https://account.ikonpass.com/api/v2/resorts";
        let resp = self.client.get(url).send()?;
        let body_str = resp.text().unwrap();
        let v: Value = serde_json::from_str(&body_str)?;
        let resorts = &v["data"].as_array().expect("API response in unexpected format");
        let mut resort_ids: HashMap<String, u64> = HashMap::new();
        for resort in resorts.iter() {
            let id = resort.as_object().unwrap().get("id").unwrap().as_u64().unwrap();
            let name = resort.as_object().unwrap().get("name").unwrap().as_str().unwrap().to_string();     
            resort_ids.insert(name, id);
        }
        Ok(resort_ids)
    }
    fn reservation_request(&self, id: u64) -> Result<String, Box<dyn std::error::Error>>  {
        let url = format!("https://account.ikonpass.com/api/v2/reservation-availability/{}", id);
        let resp = self.client.get(&url).send()?;
        let body_str = resp.text().unwrap();
        Ok(body_str)
    }

    fn get_unavailable_dates(&self, resort_ids: &HashMap<String, u64>, location: &str) -> Result<HashSet<String>, Box<dyn std::error::Error>>  { 
        let id = match resort_ids.get(location) {
            Some(id) => id,
            None => return Err(format!("Invalid location - {}", location).into())
        };
        let response_str = self.reservation_request(*id)?;
        let v: Value = serde_json::from_str(&response_str)?;
        let data = &v["data"].as_array().expect("API response in unexpected format")[0].as_object().unwrap();
        let blackout_dates = data["blackout_dates"].as_array().expect("API response in unexpected format");
        let closed_dates = data["closed_dates"].as_array().expect("API response in unexpected format");
        let unavailable_dates = data["unavailable_dates"].as_array().expect("API response in unexpected format");

        let mut all_unavailable_dates = Vec::new();
        all_unavailable_dates.extend(blackout_dates);
        all_unavailable_dates.extend(closed_dates);
        all_unavailable_dates.extend(unavailable_dates);

        let mut month_map = HashMap::new();
        month_map.insert("01", "Jan");
        month_map.insert("02", "Feb");
        month_map.insert("03", "Mar");

        let mut final_set = HashSet::new();
        let now = Utc::now();
        for date in all_unavailable_dates.iter() {
            let date_str = date.as_str().unwrap().to_string();
            let split_date: Vec<&str> = date_str.split('-').collect();
            let month = month_map.get(split_date[1]);
            if month.is_some() {
                // TODO: make this robust
                if *month.unwrap() != "Jan" || split_date[2].parse::<u32>().unwrap() > now.day() {
                    let formatted_date = format!("{} {}", month.unwrap(), split_date[2]);
                    final_set.insert(formatted_date);
                }
            }
        }
        Ok(final_set)
    }

    // TODO: move this function to a shared module for both pow snipers to use
    fn notify(&self, email_addr: &str, location: &str, date: &str) -> WebDriverResult<()> {
        let url = "https://account.ikonpass.com/en/myaccount/add-reservations/";
        let email = Message::builder()
            .from(format!("Ikon Pass Reservations <{}@gmail.com>", self.config.get_notify_username()).parse().unwrap())
            .to(format!("<{}>", email_addr).parse().unwrap())
            .subject(format!("{} Reservation", location))
            .body(format!("{} at {} is now available!\n Click the link below to reserve your spot:\n {}", date, location, url))
            .unwrap();

        let creds = Credentials::new(self.config.get_notify_username().to_string(), self.config.get_notify_password().to_string());

        // Open a remote connection to gmail
        let mailer = SmtpTransport::relay("smtp.gmail.com")
            .unwrap()
            .credentials(creds)
            .build();

        // Send the email
        match mailer.send(&email) {
            Ok(_) => log::info!("Email sent successfully to {}!", email_addr),
            Err(e) => log::error!("Could not send email: {:?}", e),
        }
        Ok(())
    }
}
