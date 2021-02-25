use crate::pi::Garage;
use mqtt_async_client::client::{Client, Publish, SubscribeTopic, QoS, ReadResult};
use mqtt_async_client::client::{Subscribe};
use mqtt_async_client::Error;

pub struct Config {
	pub name: String,
	pub doors: usize,
}

pub async fn listen(config: Config, garage: Garage) -> Result<(), Box<dyn std::error::Error>> {
	let mut builder = Client::builder();
	builder.set_host("localhost".into())
		.set_port(12u16);
	let mut client = builder.build()?;
	client.connect().await?;

	let result = client.subscribe(Subscribe::new(vec![
		SubscribeTopic { qos: QoS::ExactlyOnce, topic_path: "homeautomation/device_automation/garage/trigger".into() }
	])).await?;
	result.any_failures()?;

	client.publish(&Publish::new("homeautomation/device_automation/garage/config".into(), r#"{
"automation_type":"trigger",
"topic":"homeautomation/device_automation/garage/trigger",
"type":"button_short_press",
"subtype":"button_1",
"device":{
"connections":["mac":"00:00:00:00:00:00"]
}
}"#.as_bytes().to_vec())).await?;

	loop {
		let r = client.read_subscriptions().await;
		match r {
			Ok(_) => {}
			Err(_) => todo!()
		}
	}

	Ok(())
}
