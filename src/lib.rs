use std::vec::Vec;
use std::string::String;
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
        let file = File::open("config/settings.json").expect("Error openenign file");
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
        let file = File::open("config/info.json").expect("Error openenign file");
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

pub struct PowSniper {
    driver: WebDriver,
    config: Config,
    reservations: ReservationInfo,
    current_available: RefCell<HashSet<String>>
}
impl PowSniper {
    pub fn new(driver_url: &str, config: Config, reservations: ReservationInfo) -> WebDriverResult<PowSniper> {
        let caps = DesiredCapabilities::chrome();
        let driver = WebDriver::new(driver_url, &caps)?;
        let current_available = RefCell::new(HashSet::new());
        Ok(PowSniper { driver, config, reservations, current_available })
    }

    pub fn run(&self) -> WebDriverResult<()> {
        self.login();
        let wait = time::Duration::from_secs(5);
        let locations = self.reservations.get_locations();
        loop {
            for location in locations.iter() {
                log::debug!("Running... Checking {}", location);
                self.click_reservation_button()?;
                self.navigate_to_reservation_page(location)?;
                self.monitor_availability(location)?;
                self.redirect()?;
            }
            thread::sleep(wait);
        }
        Ok(())
    }

    fn login(&self) -> WebDriverResult<()> {
        self.driver.get(self.config.get_url())?;
        let email_input = self.driver.find_element(By::Id("email"))?;
        let password_input = self.driver.find_element(By::Id("sign-in-password"))?;
        email_input.send_keys(self.config.get_login_email())?;
        password_input.send_keys(TypingData::from(self.config.get_login_password()) + Keys::Return)?;
        Ok(())
    }

    fn click_reservation_button(&self) -> WebDriverResult<()> {
        let reservation_button = self.driver.find_element(By::XPath("//*[@id=\"root\"]/div/div/main/section[1]/div/div[1]/div/a"))?;
        reservation_button.click()?;
        Ok(())
    }

    fn navigate_to_reservation_page(&self, location: &str) -> WebDriverResult<()> {
        let location_search = self.driver.find_element(By::XPath("//*[@id=\"root\"]/div/div/main/section[2]/div/div[2]/div[2]/div[1]/div[1]/div/div/div[1]/input"))?;
        location_search.send_keys(location)?;

        self.driver.find_element(By::XPath("//*[@id=\"react-autowhatever-resort-picker-section-0-item-0\"]"))?.click()?;
        self.driver.find_element(By::XPath("//*[@id=\"root\"]/div/div/main/section[2]/div/div[2]/div[2]/div[2]/button"))?.click()?;

        Ok(())
    }
    
    fn monitor_availability(&self, location: &str) -> WebDriverResult<()> {
    //    let next_month_button = self.driver.find_element(By::XPath("//*[@id=\"root\"]/div/div/main/section[2]/div/div[2]/div[3]/div[1]/div[1]/div[1]/div/div[1]/div[2]/button[2]"))?.click()?; 

        let days = self.driver.find_elements(By::ClassName("DayPicker-Day"))?;

        for day in days.iter() {
            let curr_class = day.get_attribute("class")?.expect("No attribute named 'class'");
            let raw_available_date = day.get_attribute("aria-label")?.expect("No attribute named 'aria-label'");
            let split_dates: Vec<&str> = raw_available_date.split(' ').collect();
            let available_date = &split_dates[1..3].join(" ");
            let val = available_date.to_string() + location;
            if curr_class == "DayPicker-Day" {
                // if the val isn't in the set then send notifications if it's a day of interest
                if self.current_available.borrow_mut().insert(val.clone()) {
                    for &date in self.reservations.get_dates(location).iter() {
                        if date == available_date {
                            // if the val isn't in the set, this is a new availability and thus
                            // notifications should be sent
                            let emails = self.reservations.get_emails(location, date);
                            for email in emails.iter() {
                                log::info!("Sending email to {} for {} at {}", email, date, location);
                                self.notify(email, location, date);
                            }
                        }
                    }
                } 
            } else {
                    self.current_available.borrow_mut().remove(&val);
            }
        } 
        Ok(())
    }

    fn notify(&self, email: &str, location: &str, date: &str) -> WebDriverResult<()> {
        let url = "https://account.ikonpass.com/en/myaccount/add-reservations/";
        let email = Message::builder()
            .from(format!("testing <{}@gmail.com>", self.config.get_notify_username()).parse().unwrap())
            .to(format!("<{}>", email).parse().unwrap())
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
            Ok(_) => log::info!("Email sent successfully!"),
            Err(e) => log::error!("Could not send email: {:?}", e),
        }
        Ok(())
    }

    fn redirect(&self) -> WebDriverResult<()> {
        self.driver.get(self.config.get_url())
    }
}

