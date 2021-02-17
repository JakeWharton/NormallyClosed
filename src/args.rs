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
	#[structopt(short, long)]
	pub doors: u8,

	/// Enable debug logging
	#[structopt(long)]
	pub debug: bool,
}

pub fn parse_args() -> Args {
	Args::from_args()
}
