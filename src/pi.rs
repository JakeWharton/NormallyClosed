use crate::Door;
use async_std::sync::Arc;
use async_std::sync::Mutex;
use async_std::task;
use rppal::gpio::Gpio;
use rppal::gpio::OutputPin;
use std::error::Error;
use std::time::Duration;
use tracing::debug;

#[derive(Clone)]
pub struct Garage {
	pub doors: Vec<DoorRelay>,
}

impl Garage {
	pub fn new(gpio: Gpio, doors: Vec<Door>) -> Result<Garage, Box<dyn Error>> {
		let door_relays: Result<Vec<DoorRelay>, _> = doors
			.into_iter()
			.map(|door| {
				gpio.get(door.pin).map(|it| DoorRelay {
					door,
					pin: Arc::new(Mutex::new(it.into_output())),
				})
			})
			.collect();

		Ok(Garage {
			doors: door_relays?,
		})
	}
}

#[derive(Clone)]
pub struct DoorRelay {
	pub door: Door,
	pin: Arc<Mutex<OutputPin>>,
}

impl DoorRelay {
	pub async fn trigger(&self) {
		let pin = self.pin.clone();
		let mut pin = pin.lock().await;

		debug!("Setting pin {} HIGH", pin.pin());
		pin.set_high();

		task::sleep(Duration::from_millis(200)).await;

		debug!("Setting pin {} LOW", pin.pin());
		pin.set_low();
	}
}
