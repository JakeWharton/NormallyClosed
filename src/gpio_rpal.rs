use crate::gpio::Gpio;
use crate::gpio::GpioPin;
use rppal::gpio::Gpio as RppalGpio;
use rppal::gpio::OutputPin;
use std::error::Error;

pub struct HardwareGpio {
	gpio: RppalGpio,
}

impl HardwareGpio {
	pub fn new() -> Result<HardwareGpio, Box<dyn Error>> {
		let gpio = RppalGpio::new()?;
		Ok(HardwareGpio { gpio })
	}
}

impl Gpio for HardwareGpio {
	fn get(&self, pin: u8) -> Result<Box<dyn GpioPin>, Box<dyn Error>> {
		let output_pin = self.gpio.get(pin)?.into_output();
		let gpio_pin = HardwareGpioPin { pin: output_pin };
		Ok(Box::new(gpio_pin))
	}
}

struct HardwareGpioPin {
	pin: OutputPin,
}

impl GpioPin for HardwareGpioPin {
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
