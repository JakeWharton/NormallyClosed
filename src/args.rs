use structopt::clap::Error;
use structopt::clap::ErrorKind::InvalidValue;
use structopt::StructOpt;
use strum::VariantNames;
use strum_macros::EnumString;
use strum_macros::EnumVariantNames;

#[derive(Debug, EnumString, EnumVariantNames)]
#[strum()]
pub enum Board {
	// Pimoroni automation HAT mini with 1 relay.
	PIM213,
	// Pimoroni automation HAT with 3 relays.
	PIM487,
}

#[derive(Debug, StructOpt)]
pub struct Args {
	/// Automation HAT model
	#[structopt(short, long, possible_values = &Board::VARIANTS)]
	pub board: Board,

	/// Number of doors
	#[structopt(short, long, name = "count")]
	pub doors: usize,

	/// User-friendly names for the doors
	#[structopt(name = "NAME")]
	pub names: Vec<String>,

	/// Enable debug logging
	#[structopt(long)]
	pub debug: bool,
}

pub fn parse_args() -> Args {
	let args = Args::from_args();

	if args.names.len() > args.doors {
		Error::with_description(
			&format!(
				"Name count {} must not exceed door count {}",
				args.names.len(),
				args.doors
			),
			InvalidValue,
		)
		.exit();
	}

	args
}
