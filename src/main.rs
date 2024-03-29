use std::error::Error;
use std::fs;

use futures::join;
use structopt::clap::Error as ClapError;
use structopt::clap::ErrorKind::InvalidValue;
use tracing::debug;

use crate::config::GarageConfig;
use crate::config::RelayConfig;
use crate::garage::Garage;
use crate::gpio::Gpio;
use reqwest::Client;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Mutex;

mod cli;

mod config;

mod garage;

mod gpio;

#[cfg(feature = "rpi")]
mod gpio_rppal;

mod http;

mod sync;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
	tracing_subscriber::fmt::init();

	let args = cli::parse_args();
	debug!("{:?}", &args);

	let config = fs::read_to_string(args.config_file)?;
	let config = config::parse_config(&config)?;
	debug!("{:?}", &config);

	let local_garage = create_garage(&config)?;
	let garage = Arc::new(Mutex::new(local_garage));

	let http = http::listen(garage.clone(), args.http_port);

	let client = Arc::new(Client::builder().timeout(Duration::from_secs(10)).build()?);
	let sync = sync::poll(client, garage, config.secondary_hosts);

	let _ = join!(http, sync);

	Ok(())
}

fn create_garage(config: &GarageConfig) -> Result<Garage, Box<dyn Error>> {
	let pins = match &config.relays {
		RelayConfig::BoardBased { board } => {
			match board.as_ref() {
				// https://pinout.xyz/pinout/automation_hat_mini
				"PIM487" => vec![16u8],
				// https://pinout.xyz/pinout/automation_phat
				"PIM221" => vec![16u8],
				// https://pinout.xyz/pinout/automation_hat
				"PIM213" => vec![13u8, 19u8, 16u8],
				// https://bc-robotics.com/shop/raspberry-pi-zero-relay-hat/
				// https://bc-robotics.com/shop/raspberry-pi-zero-relay-hat-assembled/
				"RAS-109" | "RAS-194" => vec![4u8, 17u8],
				_ => {
					ClapError::with_description("Unknown board model", InvalidValue).exit();
				}
			}
		}
		RelayConfig::PinBased { pins } => pins.to_vec(),
	};
	debug!("Relays pins {:?}", &pins);

	if config.doors.is_empty() {
		ClapError::with_description("No doors defined", InvalidValue).exit();
	}

	let door_relay_count = config
		.doors
		.iter()
		.fold(0, |count, door| count + door.relay_count()) as usize;
	if door_relay_count > pins.len() {
		ClapError::with_description(
			&format!(
				"Door relay usage ({}) must not exceed available board relay count ({})",
				door_relay_count,
				pins.len(),
			),
			InvalidValue,
		)
		.exit();
	}

	let gpio = create_gpio()?;
	Garage::new(&*gpio, &config, &pins)
}

#[cfg(feature = "rpi")]
fn create_gpio() -> Result<Box<dyn Gpio>, Box<dyn Error>> {
	let gpio = gpio_rppal::RppalGpio::new()?;
	Ok(Box::new(gpio) as Box<dyn Gpio>)
}

#[allow(clippy::unnecessary_wraps)] // Maintaining source/binary compatibility with 'rpi' variant.
#[cfg(not(feature = "rpi"))]
fn create_gpio() -> Result<Box<dyn Gpio>, Box<dyn Error>> {
	Ok(Box::new(gpio::LoggingGpio::new()) as Box<dyn Gpio>)
}
