use std::error::Error;

use crate::pi::Garage;
use rppal::gpio::Gpio;

mod args;
mod http;
mod pi;

#[derive(Debug, Clone)]
pub struct Door {
	/// User-friendly name
	pub name: String,
	/// BCP GPIO pin number
	pub pin: u8,
}

#[async_std::main]
async fn main() -> Result<(), Box<dyn Error>> {
	tracing_subscriber::fmt::init();

	let doors = args::parse_garage();

	let gpio = Gpio::new()?;
	let garage = pi::create_garage(gpio, doors)?;

	http::listen(garage).await?;

	Ok(())
}
