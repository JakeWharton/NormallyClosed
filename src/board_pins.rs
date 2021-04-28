use std::error::Error;

use async_trait::async_trait;
use tokio::time::Duration;
use tracing::debug;

use crate::board::Board;
use crate::board::BoardRelay;
use crate::gpio::Gpio;
use crate::gpio::GpioPin;

pub struct PinBasedBoard {
	gpio_pins: Vec<PinBasedRelay>,
}

impl PinBasedBoard {
	pub fn new(gpio: &dyn Gpio, pins: &[u8]) -> PinBasedBoard {
		let gpio_pins: Result<Vec<PinBasedRelay>, Box<dyn Error>> = pins
			.iter()
			.map(|pin| {
				gpio.get(*pin).map(|gpio_pin| PinBasedRelay {
					pin: *pin,
					gpio_pin,
				})
			})
			.collect();
		PinBasedBoard {
			gpio_pins: gpio_pins?,
		}
	}
}

impl Board for PinBasedBoard {
	fn relays(&self) -> usize {
		self.gpio_pins.len()
	}

	fn relay(&self, index: usize) -> Box<dyn BoardRelay> {
		Box::new(self.gpio_pins[index].clone())
	}
}

struct PinBasedRelay {
	pin: u8,
	gpio_pin: Box<dyn GpioPin>,
}

#[async_trait]
impl BoardRelay for PinBasedRelay {
	async fn toggle(&mut self) {
		debug!("Setting pin {} HIGH", self.pin);
		self.gpio_pin.set_high();

		tokio::time::sleep(Duration::from_millis(200)).await;

		debug!("Setting pin {} LOW", self.pin);
		self.gpio_pin.set_low();
	}
}
