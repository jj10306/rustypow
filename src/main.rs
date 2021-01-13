use rustypow::{Config, ReservationInfo, PowSniper, ApiPowSniper};


fn main() {
    env_logger::init();
    //let bot = PowSniper::new("http://localhost:4444", config, reservations).expect("An error occurred when creating the bot");
//    let wait = time::Duration::from_secs(1 * 60);
    loop {
        let config = Config::new("config/settings.json");
        let reservations = ReservationInfo::new("path");
        let api_bot = ApiPowSniper::new(config, reservations).expect("An error occurred when creating the bot");
        if let Err(e) = api_bot.run() {
            log::error!("An error occurred: \n{}", e);
        };
    }
    
}

