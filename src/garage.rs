use async_trait::async_trait;
use std::sync::Arc;
use tokio::sync::Mutex;

#[derive(Clone)]
pub struct Garage {
	pub doors: Vec<Door>,
}

#[derive(Clone)]
pub enum Door {
	Toggle {
		name: String,
		button: Arc<Mutex<Box<dyn Button>>>,
	},
	Discrete {
		name: String,
		open_button: Arc<Mutex<Box<dyn Button>>>,
		close_button: Arc<Mutex<Box<dyn Button>>>,
		stop_button: Option<Arc<Mutex<Box<dyn Button>>>>,
	},
}

#[async_trait]
pub trait Button: Sync + Send {
	async fn trigger(&self);
}
