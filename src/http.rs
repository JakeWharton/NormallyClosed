use crate::garage::Door;
use crate::garage::Garage;
use std::convert::Infallible;
use std::net::IpAddr;
use std::net::Ipv4Addr;
use std::net::SocketAddr;
use warp::http::Uri;
use warp::Filter;
use warp::Rejection;
use warp::Reply;

fn with_garage(garage: Garage) -> impl Filter<Extract = (Garage,), Error = Infallible> + Clone {
	warp::any().map(move || garage.clone())
}

async fn index(garage: Garage) -> Result<impl Reply, Infallible> {
	let mut html = r#"<!DOCTYPE html>
<html lang="en">
<head>
<meta charset="utf-8">
<meta name="viewport" content="width=device-width,initial-scale=1">
<title>Normally Closed</title>
</head>
<body>
<h1>Normally Closed</h1>"#
		.to_string();
	for (i, door) in garage.doors.iter().enumerate() {
		match door {
			Door::Toggle { name, .. } => {
				html.push_str(&format!(
					r#"<h2>{}</h2>
		<form action="/door/{}/toggle" method="post">
		<input type="submit" value="Toggle">
		</form>"#,
					name, i
				));
			}
			Door::Discrete {
				name,
				open_button: _,
				close_button: _,
				stop_button,
			} => {
				html.push_str(&format!(
					r#"<h2>{}</h2>
		<form action="/door/{}/open" method="post">
		<input type="submit" value="Open">
		</form>
		<form action="/door/{}/close" method="post">
		<input type="submit" value="Close">
		</form>"#,
					name, i, i
				));
				if let Some(_) = stop_button {
					html.push_str(&format!(
						r#"<form action="/door/{}/stop" method="post">
		<input type="submit" value="Stop">
		</form>"#,
						i
					));
				}
			}
		};
	}
	html.push_str(
		r#"</body>
</html>"#,
	);

	Ok(warp::reply::html(html))
}

async fn toggle(id: usize, garage: Garage) -> Result<impl Reply, Rejection> {
	// TODO hoist this validation into setup
	if id >= garage.doors.len() {
		return Err(warp::reject());
	}
	let door = &garage.doors[id];
	match door {
		Door::Toggle { name: _, button } => {
			let button = button.clone();
			let button = button.lock().await;
			button.trigger().await;

			Ok(warp::redirect::see_other(Uri::from_static("/")))
		}
		Door::Discrete { .. } => Err(warp::reject()),
	}
}

async fn open(id: usize, garage: Garage) -> Result<impl Reply, Rejection> {
	if id >= garage.doors.len() {
		return Err(warp::reject());
	}
	let door = &garage.doors[id];
	match door {
		Door::Toggle { .. } => Err(warp::reject()),
		Door::Discrete {
			name: _,
			open_button,
			close_button: _,
			stop_button: _,
		} => {
			let open_button = open_button.clone();
			let open_button = open_button.lock().await;
			open_button.trigger().await;

			Ok(warp::redirect::see_other(Uri::from_static("/")))
		}
	}
}

async fn close(id: usize, garage: Garage) -> Result<impl Reply, Rejection> {
	if id >= garage.doors.len() {
		return Err(warp::reject());
	}
	let door = &garage.doors[id];
	match door {
		Door::Toggle { .. } => Err(warp::reject()),
		Door::Discrete {
			name: _,
			open_button: _,
			close_button,
			stop_button: _,
		} => {
			let close_button = close_button.clone();
			let close_button = close_button.lock().await;
			close_button.trigger().await;

			Ok(warp::redirect::see_other(Uri::from_static("/")))
		}
	}
}

async fn stop(id: usize, garage: Garage) -> Result<impl Reply, Rejection> {
	if id >= garage.doors.len() {
		return Err(warp::reject());
	}
	let door = &garage.doors[id];
	match door {
		Door::Toggle { .. } => Err(warp::reject()),
		Door::Discrete {
			name: _,
			open_button: _,
			close_button: _,
			stop_button,
		} => match stop_button {
			None => Err(warp::reject()),
			Some(stop_button) => {
				let stop_button = stop_button.clone();
				let stop_button = stop_button.lock().await;
				stop_button.trigger().await;

				Ok(warp::redirect::see_other(Uri::from_static("/")))
			}
		},
	}
}

pub async fn listen(garage: Garage, port: u16) {
	let index = warp::path::end()
		.and(warp::get())
		.and(with_garage(garage.clone()))
		.and_then(self::index);

	let door_toggle = warp::path!("door" / usize / "toggle")
		.and(warp::post())
		.and(with_garage(garage.clone()))
		.and_then(self::toggle);

	let door_open = warp::path!("door" / usize / "open")
		.and(warp::post())
		.and(with_garage(garage.clone()))
		.and_then(self::open);

	let door_close = warp::path!("door" / usize / "close")
		.and(warp::post())
		.and(with_garage(garage.clone()))
		.and_then(self::close);

	let door_stop = warp::path!("door" / usize / "stop")
		.and(warp::post())
		.and(with_garage(garage.clone()))
		.and_then(self::stop);

	let routes = index
		.or(door_toggle)
		.or(door_open)
		.or(door_close)
		.or(door_stop)
		.with(warp::trace::request());

	let address = SocketAddr::new(IpAddr::V4(Ipv4Addr::UNSPECIFIED), port);
	warp::serve(routes).run(address).await
}
