use serde::Deserialize;
use toml::de::Error;

pub fn parse_config(s: &str) -> Result<GarageConfig, Error> {
	let config = toml::from_str(s)?;

	// TODO check version is 0
	// TODO check relay indices are not used more than once

	Ok(config)
}

#[derive(Debug, PartialEq, Deserialize)]
pub struct GarageConfig {
	pub version: u32,
	#[serde(default)]
	pub secondary_hosts: Vec<String>,
	pub relays: RelayConfig,
	#[serde(rename = "door")]
	pub doors: Vec<DoorConfig>,
}

#[derive(Debug, PartialEq, Deserialize)]
#[serde(untagged)]
pub enum RelayConfig {
	BoardBased { board: String },
	PinBased { pins: Vec<u8> },
}

#[derive(Debug, PartialEq, Deserialize)]
#[serde(untagged)]
pub enum DoorConfig {
	ToggleButton {
		name: String,
		relay: usize,
	},
	DiscreteButtons {
		name: String,
		open_relay: usize,
		close_relay: usize,
		stop_relay: Option<usize>,
	},
}

impl DoorConfig {
	pub fn relay_count(&self) -> u8 {
		match self {
			DoorConfig::ToggleButton { .. } => 1,
			DoorConfig::DiscreteButtons {
				name: _,
				open_relay: _,
				close_relay: _,
				stop_relay,
			} => match stop_relay {
				None => 2,
				Some(_) => 3,
			},
		}
	}
}

#[cfg(test)]
mod test {
	use super::*;
	use crate::config::DoorConfig::DiscreteButtons;
	use crate::config::DoorConfig::ToggleButton;
	use crate::config::RelayConfig::BoardBased;
	use crate::config::RelayConfig::PinBased;

	#[test]
	fn named_board() {
		let actual = parse_config(
			r#"
			version = 0
			secondary_hosts = [
				"example.com:1234",
			]

			[relays]
			board = "example_board"

			[[door]]
			name = "Left"
			relay = 1

			[[door]]
			name = "Middle"
			open_relay = 2
			close_relay = 3

			[[door]]
			name = "Right"
			open_relay = 4
			close_relay = 5
			stop_relay = 6
			"#,
		)
		.unwrap();

		let expected = GarageConfig {
			version: 0,
			secondary_hosts: vec!["example.com:1234".to_owned()],
			relays: BoardBased {
				board: "example_board".to_string(),
			},
			doors: vec![
				ToggleButton {
					name: "Left".to_string(),
					relay: 1,
				},
				DiscreteButtons {
					name: "Middle".to_string(),
					open_relay: 2,
					close_relay: 3,
					stop_relay: None,
				},
				DiscreteButtons {
					name: "Right".to_string(),
					open_relay: 4,
					close_relay: 5,
					stop_relay: Some(6),
				},
			],
		};

		assert_eq!(actual, expected);
	}

	#[test]
	fn pins() {
		let actual = parse_config(
			r#"
			version = 0

			[relays]
			pins = [11, 13, 15, 17]

			[[door]]
			name = "Door"
			relay = 1
			"#,
		)
		.unwrap();

		let expected = GarageConfig {
			version: 0,
			secondary_hosts: vec![],
			relays: PinBased {
				pins: vec![11u8, 13u8, 15u8, 17u8],
			},
			doors: vec![ToggleButton {
				name: "Door".to_string(),
				relay: 1,
			}],
		};

		assert_eq!(actual, expected);
	}
}
