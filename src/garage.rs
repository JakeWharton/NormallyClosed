use crate::config::DoorConfig;
use crate::config::GarageConfig;
use crate::gpio::Gpio;
use crate::gpio::GpioPin;
use async_trait::async_trait;
use std::error::Error;
use std::time::Duration;
use tokio::sync::Mutex;
use tokio::time::sleep;
use tracing::debug;

pub struct Garage {
	pub doors: Vec<Door>,
}

impl Garage {
	pub fn new(
		gpio: &dyn Gpio,
		config: &GarageConfig,
		pins: &[u8],
	) -> Result<Garage, Box<dyn Error>> {
		let doors: Result<Vec<Door>, _> = config
			.doors
			.iter()
			.map(|door| match door {
				DoorConfig::ToggleButton { name, relay } => gpio.get(pins[*relay - 1]).map(|pin| {
					let button = GpioButton {
						pin: Mutex::new(pin),
					};
					Door {
						name: name.to_string(),
						host: None,
						control: DoorControl::Toggle {
							button: Box::new(button) as Box<dyn Button>,
						},
					}
				}),
				DoorConfig::DiscreteButtons {
					name,
					open_relay,
					close_relay,
					stop_relay,
				} => gpio.get(pins[*open_relay - 1]).and_then(|open_pin| {
					gpio.get(pins[*close_relay - 1]).and_then(|close_pin| {
						stop_relay
							.map(|stop_relay| gpio.get(pins[stop_relay - 1]))
							.transpose()
							.map(|stop_pin| {
								let open_button = GpioButton {
									pin: Mutex::new(open_pin),
								};
								let close_button = GpioButton {
									pin: Mutex::new(close_pin),
								};
								Door {
									name: name.to_string(),
									host: None,
									control: DoorControl::Discrete {
										open_button: Box::new(open_button) as Box<dyn Button>,
										close_button: Box::new(close_button) as Box<dyn Button>,
										stop_button: stop_pin.map(|pin| {
											let stop_button = GpioButton {
												pin: Mutex::new(pin),
											};
											Box::new(stop_button) as Box<dyn Button>
										}),
									},
								}
							})
					})
				}),
			})
			.collect();

		Ok(Garage { doors: doors? })
	}
}

pub struct Door {
	pub name: String,
	/// The secondary host which is providing this door, or None if provided locally.
	pub host: Option<String>,
	pub control: DoorControl,
}

pub enum DoorControl {
	Toggle {
		button: Box<dyn Button>,
	},
	Discrete {
		open_button: Box<dyn Button>,
		close_button: Box<dyn Button>,
		stop_button: Option<Box<dyn Button>>,
	},
}

#[async_trait]
pub trait Button: Sync + Send {
	async fn trigger(&self);
}

struct GpioButton {
	pin: Mutex<Box<dyn GpioPin>>,
}

#[async_trait]
impl Button for GpioButton {
	async fn trigger(&self) {
		let mut pin = self.pin.lock().await;

		debug!("Setting pin {} HIGH", pin.pin());
		pin.set_high();

		sleep(Duration::from_millis(200)).await;

		debug!("Setting pin {} LOW", pin.pin());
		pin.set_low();
	}
}
