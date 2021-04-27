use crate::config::DoorConfig;
use crate::config::GarageConfig;
use crate::gpio::Gpio;
use crate::gpio::GpioPin;
use async_trait::async_trait;
use std::error::Error;
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio::time::Duration;
use tracing::debug;

#[derive(Clone)]
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
					Door::Toggle {
						name: name.to_string(),
						button: Arc::new(Box::new(button) as Box<dyn Button>),
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
								Door::Discrete {
									name: name.to_string(),
									open_button: Arc::new(Box::new(open_button) as Box<dyn Button>),
									close_button: Arc::new(Box::new(close_button) as Box<dyn Button>),
									stop_button: stop_pin.map(|pin| {
										let stop_button = GpioButton {
											pin: Mutex::new(pin),
										};
										Arc::new(Box::new(stop_button) as Box<dyn Button>)
									}),
								}
							})
					})
				}),
			})
			.collect();

		Ok(Garage { doors: doors? })
	}
}

#[derive(Clone)]
pub enum Door {
	Toggle {
		name: String,
		button: Arc<Box<dyn Button>>,
	},
	Discrete {
		name: String,
		open_button: Arc<Box<dyn Button>>,
		close_button: Arc<Box<dyn Button>>,
		stop_button: Option<Arc<Box<dyn Button>>>,
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

		tokio::time::sleep(Duration::from_millis(200)).await;

		debug!("Setting pin {} LOW", pin.pin());
		pin.set_low();
	}
}
