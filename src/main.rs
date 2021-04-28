use std::error::Error;
use std::fs;

use structopt::clap::Error as ClapError;
use structopt::clap::ErrorKind::InvalidValue;
use tracing::debug;

use crate::board_pins::PinBasedBoard;
use crate::config::GarageConfig;
use crate::config::RelayConfig;
use crate::garage::Garage;
use crate::gpio::Gpio;

mod board;

mod board_pins;

mod cli;

mod config;

mod garage;

mod gpio;

#[cfg(feature = "rpi")]
mod gpio_rppal;

mod http;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
	tracing_subscriber::fmt::init();

	let args = cli::parse_args();
	debug!("{:?}", &args);

	let config = fs::read_to_string(args.config_file)?;
	let config = config::parse_config(&config)?;
	debug!("{:?}", &config);

	let garage = create_garage(&config)?;

	http::listen(garage, args.http_port).await;
	Ok(())
}

fn create_garage(config: &GarageConfig) -> Result<Garage, Box<dyn Error>> {
	let gpio = create_gpio()?;
	let board = match &config.relays {
		RelayConfig::BoardBased { board } => match board::from_name(&gpio, board) {
			None => {
				ClapError::with_description("Unknown board model", InvalidValue).exit();
			}
			Some(board) => board,
		},
		RelayConfig::PinBased { pins } => PinBasedBoard::new(&gpio, pins),
	};

	if config.doors.is_empty() {
		ClapError::with_description("No doors defined", InvalidValue).exit();
	}

	let door_relay_count = config
		.doors
		.iter()
		.fold(0, |count, door| count + door.relay_count()) as usize;
	if door_relay_count > board.relays() {
		ClapError::with_description(
			&format!(
				"Door relay usage ({}) must not exceed available board relay count ({})",
				door_relay_count,
				board.relays(),
			),
			InvalidValue,
		)
		.exit();
	}

	Garage::new(&config, &board)
}

#[cfg(feature = "rpi")]
fn create_gpio() -> Result<Box<dyn Gpio>, Box<dyn Error>> {
	let gpio = gpio_rppal::RppalGpio::new()?;
	Ok(Box::new(gpio) as Box<dyn Gpio>)
}

#[cfg(not(feature = "rpi"))]
fn create_gpio() -> Result<Box<dyn Gpio>, Box<dyn Error>> {
	Ok(Box::new(gpio::LoggingGpio::new()) as Box<dyn Gpio>)
}
