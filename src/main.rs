use rustypow::{Config, ReservationInfo, PowSniper};
use std::{thread, time};



fn main() {
    
    let wait = time::Duration::from_secs(10);
    loop {
        let config = Config::new("path");
        let reservations = ReservationInfo::new("path");
        let bot = PowSniper::new("http://localhost:4444", config, reservations).expect("An error occurred when creating the bot");
        bot.run();
        thread::sleep(wait);
    }
}

