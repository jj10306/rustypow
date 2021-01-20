mod utils;
mod powsniper;

use powsniper::ApiPowSniper;
use utils::{ ReservationInfo, Config };


fn main() {
    env_logger::init();
    loop {
        let config = Config::new("config/settings.json");
        let reservations = ReservationInfo::new("config/info.json");
        let api_bot = ApiPowSniper::new(config, reservations).expect("An error occurred when creating the bot");
        if let Err(e) = api_bot.run() {
            log::error!("An error occurred: \n{}", e);
        };
    }
    
}

