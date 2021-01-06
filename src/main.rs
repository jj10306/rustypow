use rustypow::{Config, ReservationInfo, PowSniper, ApiPowSniper};


fn main() {
    env_logger::init();
    let config = Config::new("path");
    let reservations = ReservationInfo::new("path");
    //let bot = PowSniper::new("http://localhost:4444", config, reservations).expect("An error occurred when creating the bot");
    let api_bot = ApiPowSniper::new( config, reservations).expect("An error occurred when creating the bot");
    if let Err(e) = api_bot.run() {
        log::error!("An error occurred: \n{}", e);
    };
}

