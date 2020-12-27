use thirtyfour_sync::prelude::*;
use std::vec::Vec;
use std::string::String;
use std::collections::HashMap;
use std::{thread, time};

use serde::Deserialize;
use serde_json::Value;
use std::error::Error;
use std::fs::File;
use std::io::BufReader;
use std::path::Path;

#[derive(Deserialize)]
pub struct Config {
    url: String,
    login_email: String,
    login_password: String,
    notify_email: String,
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

    fn get_notify_email(&self) -> &str {
        &self.notify_email
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
                                    println!("{} {} {}", email, location, date);
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
    pub reservations: ReservationInfo
}
impl PowSniper {
    pub fn new(driver_url: &str, config: Config, reservations: ReservationInfo) -> WebDriverResult<PowSniper> {
        let caps = DesiredCapabilities::chrome();
        let driver = WebDriver::new(driver_url, &caps)?;
        Ok(PowSniper { driver, config, reservations })
    }

    pub fn run(&self) {
        let locations = self.reservations.get_locations();
        let wait = time::Duration::from_secs(10);
        while true {
            for location in locations.iter() {
                self.login();
                self.click_reservation_button();
                self.navigate_to_reservation_page(location);
                self.monitor_availability(location);
            }
            thread::sleep(wait);
        }
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
        let next_month_button = self.driver.find_element(By::XPath("//*[@id=\"root\"]/div/div/main/section[2]/div/div[2]/div[3]/div[1]/div[1]/div[1]/div/div[1]/div[2]/button[2]"))?.click()?; 

        let days = self.driver.find_elements(By::ClassName("DayPicker-Day"))?;

        for day in days.iter() {
            let curr_class = day.get_attribute("class")?.expect("No attribute named 'class'");
            if curr_class == "DayPicker-Day" {
                let available_date = day.get_attribute("aria-label")?.expect("No attribute named 'aria-label'");
                println!("{:?}", available_date);
                for date in self.reservations.get_dates(location).iter() {
                   println!("{:?}", date); 
                }
            }
        }
        Ok(())
    }
//    def monitor_availability(dates, location):
//        # go to january
//        next_month_button = wait_for_element_by_xpath('//*[@id="root"]/div/div/main/section[2]/div/div[2]/div[3]/div[1]/div[1]/div[1]/div/div[1]/div[2]/button[2]')
//        next_month_button.click()
//
//        days = driver.find_elements_by_class_name("DayPicker-Day") 
//        for day in days:
//            curr_class = day.get_attribute("class")
//            # this doesn't include the day of because that element has an additional class
//            if curr_class == "DayPicker-Day":
//                # available date is of the form: Jan 22
//                available_date = " ".join(day.get_attribute("aria-label").split()[1:3])
//                for date in dates:
//                    if date == available_date:
//                        message = f"{available_date} is available at {location}"
//                        notify.send_email("jjohnson473@gatech.edu", message)
//                        notify.send_email("zakcho25@gmail.com ", message)
//                        logging.info(message)
   
    
}
