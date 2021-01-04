use rustypow::{Config, ReservationInfo, PowSniper};


fn main() {
    env_logger::init();
    let config = Config::new("path");
    let reservations = ReservationInfo::new("path");
    let bot = PowSniper::new("http://localhost:4444", config, reservations).expect("An error occurred when creating the bot");
    if let Err(e) = bot.run() {
        log::error!("An error occurred: \n{}", e);
    };
}

