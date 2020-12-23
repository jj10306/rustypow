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
struct Config {
    url: String,
    login_email: String,
    login_password: String,
    notify_email: String,
    notify_password: String 
}
impl Config {
    fn new(path: &str) -> Config {
        // Open the file in read-only mode with buffer.
        let file = File::open("config/settings.json").expect("Error openenign file");
        let reader = BufReader::new(file);

        // Read the JSON contents of the file as an instance of `User`.
        let u = serde_json::from_reader(reader).expect("Error when parsing json");
        u

     //   let url = "https://account.ikonpass.com/en/login?redirect_uri=/en/myaccount";
     //   let login_email = "johnsonjakob99@gmail.com";
     //   let login_password = "Johnson99";
     //   let notify_email = "powsniper99@gmail.com";
     //   let notify_password = "POW_sniper99";
     //   Config { url, login_email, login_password, notify_email, notify_password }
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

struct ReservatioInfo {
    //inner: HashMap<String, HashMap<String, Vec<String>>>
    a:u32
}
impl ReservatioInfo {
    fn new(path: &str) -> ReservatioInfo {
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
                                    let date = date.as_str().unwrap();
                                    println!("{} {} {}", email, location, date);
                                }
                            } 
                        },
                        _ => panic!("should have been an object")
                    }
                }
            },
            _ => panic!("should have been an object")
        }
        
        
        ReservatioInfo {a:1} 
    }
}



struct PowSniper {
    driver: WebDriver,
    config: Config,
    reservations: ReservatioInfo
}
impl PowSniper {
    fn new(driver_url: &str, config: Config, reservations: ReservatioInfo) -> WebDriverResult<PowSniper> {
        let caps = DesiredCapabilities::chrome();
        let driver = WebDriver::new(driver_url, &caps)?;
        Ok(PowSniper { driver, config, reservations })
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
}




fn main() -> WebDriverResult<()> {
    let config = Config::new("path");
    let reservations = ReservatioInfo::new("path");
    let bot = PowSniper::new("http://localhost:4444", config, reservations)?;

    bot.login();
    bot.click_reservation_button();
    bot.navigate_to_reservation_page("Brighton");

    let wait = time::Duration::from_secs(10);
    thread::sleep(wait);

    Ok(())
}

fn login(email: &str, password: &str) -> WebDriverResult<()> {
    Ok(())
}