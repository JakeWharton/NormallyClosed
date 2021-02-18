use structopt::clap::Error;
use structopt::clap::ErrorKind::InvalidValue;
use structopt::StructOpt;
use strum::VariantNames;
use strum_macros::EnumString;
use strum_macros::EnumVariantNames;
use tracing::debug;

use crate::Door;

#[derive(Debug, EnumString, EnumVariantNames)]
#[strum()]
enum Board {
	// Pimoroni automation HAT mini with 1 relay.
	PIM213,
	// Pimoroni automation pHAT with 1 relay.
	PIM221,
	// Pimoroni automation HAT with 3 relays.
	PIM487,
}

#[derive(Debug, StructOpt)]
struct Args {
	/// Automation HAT model
	#[structopt(short, long, name = "model", possible_values = &Board::VARIANTS)]
	board: Board,

	/// Number of doors
	#[structopt(short, long, name = "count")]
	doors: usize,

	/// User-friendly names for the doors
	#[structopt(name = "NAME")]
	names: Vec<String>,
}

pub fn parse_doors() -> Vec<Door> {
	let args: Args = Args::from_args();
	debug!("{:?}", &args);

	let relays = match args.board {
		// https://pinout.xyz/pinout/automation_hat_mini
		Board::PIM213 => vec![16u8],
		// https://pinout.xyz/pinout/automation_phat
		Board::PIM221 => vec![16u8],
		// https://pinout.xyz/pinout/automation_hat
		Board::PIM487 => vec![13u8, 19u8, 16u8],
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

	doors
}
