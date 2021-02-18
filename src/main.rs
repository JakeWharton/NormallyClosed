use std::time::Duration;

use async_std::sync::Arc;
use async_std::sync::Mutex;
use async_std::task;
use rppal::gpio::Gpio;
use rppal::gpio::OutputPin;
use structopt::clap::Error;
use structopt::clap::ErrorKind::InvalidValue;
use tide::http::mime;
use tide::Redirect;
use tide::Response;
use tide::StatusCode;
use tide_tracing::TraceMiddleware;
use tracing::debug;
use tracing::Level;

use args::Board;

mod args;

const GPIO_RELAY_1: u8 = 13;
const GPIO_RELAY_2: u8 = 19;
const GPIO_RELAY_3: u8 = 16;

#[async_std::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
	let args = args::parse_args();

	let subscriber = tracing_subscriber::fmt()
		.with_max_level(if args.debug {
			Level::DEBUG
		} else {
			Level::INFO
		})
		.finish();

	tracing::subscriber::set_global_default(subscriber).expect("no global subscriber has been set");

	debug!("{:?}", &args);

	let relays = match args.board {
		Board::PIM213 => vec![GPIO_RELAY_3],
		Board::PIM487 => vec![GPIO_RELAY_1, GPIO_RELAY_2, GPIO_RELAY_3],
	};
	debug!("Relays pins {:?}", &relays);

	if args.doors < 1 || args.doors > relays.len() {
		Error::with_description(
			&format!(
				"Door count {} must not exceed available board relay count {}",
				args.doors,
				relays.len()
			),
			InvalidValue,
		)
		.exit();
	}

	let gpio = Gpio::new()?;
	let pins: Result<Vec<OutputPin>, _> = relays
		.into_iter()
		.take(args.doors)
		.map(|pin| gpio.get(pin).map(|it| it.into_output()))
		.collect();

	let mut app = tide::with_state(State {
		doors: args.doors,
		names: args.names,
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
		for i in 1..=state.doors {
			let name = if i <= state.names.len() {
				state.names[i - 1].to_string()
			} else {
				format!("Door {}", i)
			};
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
			if id < 1 || id > state.pins.len() {
				return Ok(Response::new(StatusCode::BadRequest));
			}

			let pin_arc = state.pins[id - 1].clone();
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
	doors: usize,
	names: Vec<String>,
	pins: Vec<Arc<Mutex<OutputPin>>>,
}
