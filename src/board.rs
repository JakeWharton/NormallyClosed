pub trait Board {
	fn relays(&self) -> usize;
	fn relay(&self, index: usize) -> dyn BoardRelay;
}

impl dyn Board {
	fn from_name(name: &str) -> Option<Box<dyn Board>> {
		let pins = match name {
			// https://pinout.xyz/pinout/automation_hat_mini
			"PIM487" => vec![16u8],
			// https://pinout.xyz/pinout/automation_phat
			"PIM221" => vec![16u8],
			// https://pinout.xyz/pinout/automation_hat
			"PIM213" => vec![13u8, 19u8, 16u8],
			// https://bc-robotics.com/shop/raspberry-pi-zero-relay-hat/
			// https://bc-robotics.com/shop/raspberry-pi-zero-relay-hat-assembled/
			"RAS-109" | "RAS-194" => vec![4u8, 17u8],
			_ => {
				None
			}
		};
	}
}

#[async_trait]
pub trait BoardRelay {
	async fn toggle(&self);
}
