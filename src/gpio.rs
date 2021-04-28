use std::error::Error;

pub trait Gpio {
	fn get(&self, pin: u8) -> Result<Box<dyn GpioPin>, Box<dyn Error>>;
}

pub trait GpioPin: Send + Sync {
	fn pin(&self) -> u8;
	fn set_high(&mut self);
	fn set_low(&mut self);
}

pub struct LoggingGpio {}

impl LoggingGpio {
	#[allow(dead_code)] // Only used when default features are disabled.
	pub fn new() -> LoggingGpio {
		LoggingGpio {}
	}
}

impl Gpio for LoggingGpio {
	fn get(&self, pin: u8) -> Result<Box<dyn GpioPin>, Box<dyn Error>> {
		let gpio_pin = LoggingGpioPin { pin };
		Ok(Box::new(gpio_pin))
	}
}

struct LoggingGpioPin {
	pin: u8,
}

impl GpioPin for LoggingGpioPin {
	fn pin(&self) -> u8 {
		self.pin
	}

	fn set_high(&mut self) {
		println!("GPIO {} HIGH", self.pin)
	}

	fn set_low(&mut self) {
		println!("GPIO {} LOW", self.pin)
	}
}
