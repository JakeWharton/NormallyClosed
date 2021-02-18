use std::error::Error;
use std::time::Duration;

use async_std::sync::Arc;
use async_std::sync::Mutex;
use async_std::task;
use rppal::gpio::Gpio;
use rppal::gpio::OutputPin;
use tide::http::mime;
use tide::Redirect;
use tide::Response;
use tide::StatusCode;
use tide_tracing::TraceMiddleware;
use tracing::debug;

mod args;

#[derive(Debug, Clone)]
pub struct Door {
	/// User-friendly name
	pub name: String,
	/// BCP GPIO pin number
	pub pin: u8,
}

#[derive(Debug, Clone)]
pub struct Garage {
	pub doors: Vec<Door>,
}

#[async_std::main]
async fn main() -> Result<(), Box<dyn Error>> {
	tracing_subscriber::fmt::init();

	let garage = args::parse_garage();

	let gpio = Gpio::new()?;
	let pins: Result<Vec<OutputPin>, _> = garage
		.doors
		.iter()
		.map(|door| gpio.get(door.pin).map(|it| it.into_output()))
		.collect();

	let mut app = tide::with_state(State {
		doors: garage.doors,
		pins: pins?
			.into_iter()
			.map(|pin| Arc::new(Mutex::new(pin)))
			.collect(),
	});
	app.with(TraceMiddleware::new());

	app.at("/").get(|req: tide::Request<State>| async move {
		let state = req.state();

		let mut html = r#"<!DOCTYPE html>
<html lang="en">
<head>
<meta charset="utf-8">
<meta name="viewport" content="width=device-width,initial-scale=1">
<title>Garage Pie</title>
</head>
<body>
<h1>Garage Pie</h1>"#
			.to_string();
		for i in 0..state.pins.len() {
			let name = &state.doors[i].name;
			html.push_str(&format!(
				r#"<form action="/door/{}" method="post">
<label>{} <input type="submit" value="Toggle"></label>
</form>"#,
				i, name
			));
		}
		html.push_str(
			r#"</body>
</html>"#,
		);

		Ok(Response::builder(200).body(html).content_type(mime::HTML))
	});

	app
		.at("/door/:id")
		.post(|req: tide::Request<State>| async move {
			let state = req.state();

			let id: usize = req.param("id")?.parse()?;
			if id >= state.pins.len() {
				return Ok(Response::new(StatusCode::BadRequest));
			}

			let pin_arc = state.pins[id].clone();
			let mut pin = pin_arc.lock().await;
			debug!("Setting pin {} HIGH", pin.pin());
			pin.set_high();

			task::sleep(Duration::from_millis(200)).await;

			debug!("Setting pin {} LOW", pin.pin());
			pin.set_low();

			Ok(Redirect::new("/").into())
		});

	app.listen("0.0.0.0:8080").await?;

	Ok(())
}

#[derive(Clone)]
struct State {
	doors: Vec<Door>,
	pins: Vec<Arc<Mutex<OutputPin>>>,
}
