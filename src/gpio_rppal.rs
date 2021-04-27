use crate::gpio::Gpio;
use crate::gpio::GpioPin;
use rppal::gpio::Gpio as RealGpio;
use rppal::gpio::OutputPin;
use std::error::Error;

pub struct RppalGpio {
	gpio: RealGpio,
}

impl RppalGpio {
	pub fn new() -> Result<RppalGpio, Box<dyn Error>> {
		let gpio = RealGpio::new()?;
		Ok(RppalGpio { gpio })
	}
}

impl Gpio for RppalGpio {
	fn get(&self, pin: u8) -> Result<Box<dyn GpioPin>, Box<dyn Error>> {
		let output_pin = self.gpio.get(pin)?.into_output();
		let gpio_pin = RppalGpioPin { pin: output_pin };
		Ok(Box::new(gpio_pin))
	}
}

struct RppalGpioPin {
	pin: OutputPin,
}

impl GpioPin for RppalGpioPin {
	fn pin(&self) -> u8 {
		self.pin.pin()
	}

	fn set_high(&mut self) {
		self.pin.set_high()
	}

	fn set_low(&mut self) {
		self.pin.set_low()
	}
}
