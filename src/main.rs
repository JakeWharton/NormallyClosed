use structopt::clap::ArgGroup;
use structopt::clap::Error;
use structopt::clap::ErrorKind::InvalidValue;
use structopt::StructOpt;
use tracing::debug;

use crate::pi::Garage;

mod http;
mod mqtt;
mod pi;

#[derive(Debug, Clone)]
pub struct Door {
	/// User-friendly name
	pub name: String,
	/// BCP GPIO pin number
	pub pin: u8,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
	tracing_subscriber::fmt::init();

	let args: Args = Args::from_args();
	debug!("{:?}", &args);

	let relays = if let Some(board) = args.board {
		if !args.gpio.is_empty() {
			Error::with_description("Specify one of --board or --gpio, not both", InvalidValue).exit();
		}
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
				Error::with_description("Unknown board model", InvalidValue).exit();
			}
		}
	} else {
		if args.gpio.is_empty() {
			Error::with_description("One of --board or --gpio must be specified", InvalidValue).exit();
		}
		args.gpio
	};
	debug!("Relays pins {:?}", &relays);

	if args.doors < 1 || args.doors > relays.len() {
		Error::with_description(
			&format!(
				"Door count ({}) must not exceed available board relay count ({})",
				args.doors,
				relays.len()
			),
			InvalidValue,
		)
		.exit();
	}

	if args.names.len() > args.doors {
		Error::with_description(
			&format!(
				"Name count ({}) must not exceed door count ({})",
				args.names.len(),
				args.doors
			),
			InvalidValue,
		)
		.exit();
	}

	let mut doors = vec![];
	for i in 0..args.doors {
		doors.push(Door {
			name: args
				.names
				.get(i)
				.map(|name| name.to_string())
				.unwrap_or_else(|| format!("Door {}", i + 1)),
			pin: relays[i],
		});
	}

	let garage = Garage::new(doors)?;

	let http_config = http::Config {
		port: args.http_port,
	};
	let http = http::listen(http_config, garage.clone());

	let mqtt_config = mqtt::Config {
	};
	let mqtt = mqtt::listen(mqtt_config, garage);

	let (_, mqtt) = tokio::join!(http, mqtt);
	mqtt?;

	Ok(())
}

#[derive(Debug, StructOpt)]
#[structopt(group = ArgGroup::with_name("pins").required(true))]
struct Args {
	/// Manufactured HAT board with preconfigured relays
	///
	/// PIM213 - Pimoroni automation HAT mini with 1 relay
	///{n}PIM221 - Pimoroni automation pHAT with 1 relay
	///{n}PIM487 - Pimoroni automation HAT with 3 relays
	///{n}RAS-109 - BC Robotics relay HAT with 2 relays.
	///{n}RAS-194 - BC Robotics relay HAT (assembled) with 2 relays.
	///{n}
	#[structopt(long, name = "model", group = "pins")]
	board: Option<String>,

	/// Custom BCP GPIO pin number to trigger a door relay
	#[structopt(long, name = "pin", group = "pins")]
	gpio: Vec<u8>,

	/// Number of doors
	#[structopt(long, name = "count")]
	doors: usize,

	/// User-friendly names for the doors
	#[structopt(name = "NAME")]
	names: Vec<String>,

	/// HTTP port
	#[structopt(long, name = "port", default_value = "31415")]
	http_port: u16,
}
