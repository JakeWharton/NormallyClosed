use std::path::PathBuf;
use structopt::StructOpt;

pub fn parse_args() -> Args {
	Args::from_args()
}

#[derive(Debug, StructOpt)]
#[structopt()]
pub struct Args {
	/// TOML configuration file. See https://github.com/JakeWharton/NormallyClosed#configuration
	#[structopt(parse(from_os_str))]
	pub config_file: PathBuf,

	/// HTTP port
	#[structopt(long, name = "port", default_value = "31415")]
	pub http_port: u16,
}
