use rustypow::{Config, ReservationInfo, PowSniper};



fn main() {
    let config = Config::new("path");
    let reservations = ReservationInfo::new("path");
    let bot = PowSniper::new("http://localhost:4444", config, reservations).expect("An error occurred when creating the bot");
    
    bot.run();
}

