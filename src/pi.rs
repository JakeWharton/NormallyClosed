use std::error::Error;
use std::sync::Arc;
use std::time::Duration;

use async_trait::async_trait;
use rppal::gpio::Gpio;
use rppal::gpio::OutputPin;
use tokio::sync::Mutex;
use tracing::debug;

use crate::config::DoorConfig;
use crate::config::GarageConfig;
use crate::garage::Button;
use crate::garage::Door;
use crate::garage::Garage;

pub fn create_garage(config: &GarageConfig, pins: &Vec<u8>) -> Result<Garage, Box<dyn Error>> {
	let gpio = Gpio::new()?;

	let doors: Result<Vec<Door>, _> = config
		.doors
		.iter()
		.map(|door| match door {
			DoorConfig::ToggleButton { name, relay } => gpio.get(pins[*relay - 1]).map(|pin| {
				let button = GpioButton {
					pin: Arc::new(Mutex::new(pin.into_output())),
				};
				Door::Toggle {
					name: name.to_string(),
					button: Arc::new(Mutex::new(Box::new(button))),
				}
			}),
			DoorConfig::DiscreteButtons {
				name,
				open_relay,
				close_relay,
				stop_relay,
			} => gpio.get(pins[*open_relay - 1]).and_then(|open_pin| {
				let open_button = GpioButton {
					pin: Arc::new(Mutex::new(open_pin.into_output())),
				};
				gpio.get(pins[*close_relay - 1]).and_then(|close_pin| {
					let close_button = GpioButton {
						pin: Arc::new(Mutex::new(close_pin.into_output())),
					};
					stop_relay
						.map(|stop_relay| {
							gpio.get(pins[stop_relay - 1]).map(|stop_pin| GpioButton {
								pin: Arc::new(Mutex::new(stop_pin.into_output())),
							})
						})
						.transpose()
						.map(|stop_button| Door::Discrete {
							name: name.to_string(),
							open_button: Arc::new(Mutex::new(Box::new(open_button) as Box<dyn Button>)),
							close_button: Arc::new(Mutex::new(Box::new(close_button) as Box<dyn Button>)),
							stop_button: stop_button
								.map(|button| Arc::new(Mutex::new(Box::new(button) as Box<dyn Button>))),
						})
				})
			}),
		})
		.collect();

	Ok(Garage { doors: doors? })
}

struct GpioButton {
	pin: Arc<Mutex<OutputPin>>,
}

#[async_trait]
impl Button for GpioButton {
	async fn trigger(&self) {
		let pin = self.pin.clone();
		let mut pin = pin.lock().await;

		debug!("Setting pin {} HIGH", pin.pin());
		pin.set_high();

		tokio::time::sleep(Duration::from_millis(200)).await;

		debug!("Setting pin {} LOW", pin.pin());
		pin.set_low();
	}
}
