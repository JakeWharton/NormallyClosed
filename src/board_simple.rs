use crate::board::{Board, BoardRelay};

pub struct SimpleBoard {
}

impl SimpleBoard {
	pub fn new(pins: &[u8]) -> SimpleBoard {
		todo!()
	}
}

impl Board for SimpleBoard {
	fn relays(&self) -> usize {
		todo!()
	}

	fn relay(&self, index: usize) -> Box<dyn BoardRelay> {
		todo!()
	}
}
