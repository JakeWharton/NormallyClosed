use crate::board::Board;
use crate::board::BoardRelay;
use crate::config::DoorConfig;
use crate::config::GarageConfig;
use std::error::Error;
use std::sync::Arc;

#[derive(Clone)]
pub struct Garage {
	pub doors: Vec<Door>,
}

impl Garage {
	pub fn new(config: &GarageConfig, board: &impl Board) -> Result<Garage, Box<dyn Error>> {
		let doors: Vec<Door> = config
			.doors
			.iter()
			.map(|door| match door {
				DoorConfig::ToggleButton { name, relay } => {
					let relay1 = board.relay(*relay - 1);
					Door::Toggle {
						name: name.to_string(),
						relay: Arc::new(relay1),
					}
				}
				DoorConfig::DiscreteButtons {
					name,
					open_relay,
					close_relay,
					stop_relay,
				} => {
					let open_relay1 = board.relay(*open_relay - 1);
					let close_relay1 = board.relay(*close_relay - 1);
					let stop_relay1 = stop_relay.map(|stop_relay| board.relay(stop_relay - 1));
					Door::Discrete {
						name: name.to_string(),
						open_relay: Arc::new(open_relay1),
						close_relay: Arc::new(close_relay1),
						stop_relay: stop_relay1.map(|relay| Arc::new(relay)),
					}
				}
			})
			.collect();

		Ok(Garage { doors })
	}
}

#[derive(Clone)]
pub enum Door {
	Toggle {
		name: String,
		relay: Arc<dyn BoardRelay>,
	},
	Discrete {
		name: String,
		open_relay: Arc<dyn BoardRelay>,
		close_relay: Arc<dyn BoardRelay>,
		stop_relay: Option<Arc<dyn BoardRelay>>,
	},
}
