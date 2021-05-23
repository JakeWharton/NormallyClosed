use crate::garage::Button;
use crate::garage::Door;
use crate::garage::DoorControl;
use crate::garage::Garage;
use async_trait::async_trait;
use reqwest::Client;
use serde::Deserialize;
use serde::Serialize;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Mutex;
use tokio::time::sleep;

pub async fn poll(client: Arc<Client>, garage: Arc<Mutex<Garage>>, hosts: Vec<String>) {
	if hosts.is_empty() {
		return; // Nothing to do!
	}

	// Doors are not cloneable, and instead of forcing them to be we can simply truncate the Vec
	// down to its original size after each sync.
	let garage_lock = garage.lock().await;
	let local_doors_count = garage_lock.doors.len();
	drop(garage_lock);

	loop {
		let mut remote_doors: Vec<Door> = vec![];

		for host in &hosts {
			let response = client.get(format!("{}/doors.json", host)).send().await;
			match response {
				Ok(response) => {
					let json = response.json::<GarageJson>().await;
					match json {
						Ok(other_garage) => {
							// TODO validate versions match

							for (i, other_door) in other_garage.doors.iter().enumerate() {
								let new_door = match other_door {
									DoorJson::Toggle { name } => {
										let button = RemoteButton {
											client: client.clone(),
											url: format!("{}/door/{}/toggle", host, i),
										};
										Door {
											name: name.to_string(),
											host: Some(host.to_string()),
											control: DoorControl::Toggle {
												button: Box::new(button) as Box<dyn Button>,
											},
										}
									}
									DoorJson::Discrete { name, stop_button } => {
										let open_button = RemoteButton {
											client: client.clone(),
											url: format!("{}/door/{}/open", host, i),
										};
										let close_button = RemoteButton {
											client: client.clone(),
											url: format!("{}/door/{}/close", host, i),
										};
										let stop_button = if *stop_button {
											Some(RemoteButton {
												client: client.clone(),
												url: format!("{}/door/{}/stop", host, i),
											})
										} else {
											None
										};
										Door {
											name: name.to_string(),
											host: Some(host.to_string()),
											control: DoorControl::Discrete {
												open_button: Box::new(open_button),
												close_button: Box::new(close_button),
												stop_button: stop_button.map(|button| Box::new(button) as Box<dyn Button>),
											},
										}
									}
								};
								remote_doors.push(new_door);
							}
						}
						Err(e) => {
							eprintln!("{}", e);
						}
					};
				}
				Err(e) => {
					eprintln!("{}", e);
				}
			};
		}

		let mut garage_lock = garage.lock().await;
		garage_lock.doors.truncate(local_doors_count);
		garage_lock.doors.append(&mut remote_doors);
		drop(garage_lock);

		sleep(Duration::from_secs(60)).await;
	}
}

pub fn garage_json(garage: &Garage) -> impl Serialize {
	let json_doors: Vec<DoorJson> = garage
		.doors
		.iter()
		.map(|door| match &door.control {
			DoorControl::Toggle { .. } => DoorJson::Toggle {
				name: door.name.to_string(),
			},
			DoorControl::Discrete {
				open_button: _,
				close_button: _,
				stop_button,
			} => DoorJson::Discrete {
				name: door.name.to_string(),
				stop_button: stop_button.is_some(),
			},
		})
		.collect();

	GarageJson {
		version: 0,
		doors: json_doors,
	}
}

#[derive(Debug, Serialize, Deserialize)]
struct GarageJson {
	version: u8,
	doors: Vec<DoorJson>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(untagged)]
enum DoorJson {
	Discrete { name: String, stop_button: bool },
	Toggle { name: String },
}

struct RemoteButton {
	client: Arc<Client>,
	url: String,
}

#[async_trait]
impl Button for RemoteButton {
	async fn trigger(&self) {
		let res = self.client.post(&self.url).send().await;
		if let Err(e) = res {
			eprintln!("{}", e);
		}
		// TODO propagate failure back to UI.
	}
}
