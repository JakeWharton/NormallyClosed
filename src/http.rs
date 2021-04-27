use crate::garage::Button;
use crate::garage::Door;
use crate::garage::Garage;
use std::convert::Infallible;
use std::net::IpAddr;
use std::net::Ipv4Addr;
use std::net::SocketAddr;
use std::sync::Arc;
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

async fn lookup_door(id: usize, garage: Garage) -> Result<Door, Rejection> {
	if id >= garage.doors.len() {
		return Err(warp::reject());
	}
	let door = &garage.doors[id];
	Ok(door.to_owned())
}

async fn extract_toggle_button(door: Door) -> Result<Arc<Box<dyn Button>>, Rejection> {
	match door {
		Door::Toggle { name: _, button } => Ok(button),
		Door::Discrete { .. } => Err(warp::reject()),
	}
}

async fn extract_open_button(door: Door) -> Result<Arc<Box<dyn Button>>, Rejection> {
	match door {
		Door::Toggle { .. } => Err(warp::reject()),
		Door::Discrete {
			name: _,
			open_button,
			close_button: _,
			stop_button: _,
		} => Ok(open_button),
	}
}

async fn extract_close_button(door: Door) -> Result<Arc<Box<dyn Button>>, Rejection> {
	match door {
		Door::Toggle { .. } => Err(warp::reject()),
		Door::Discrete {
			name: _,
			open_button: _,
			close_button,
			stop_button: _,
		} => Ok(close_button),
	}
}

async fn extract_stop_button(door: Door) -> Result<Arc<Box<dyn Button>>, Rejection> {
	match door {
		Door::Toggle { .. } => Err(warp::reject()),
		Door::Discrete {
			name: _,
			open_button: _,
			close_button: _,
			stop_button,
		} => match stop_button {
			None => Err(warp::reject()),
			Some(stop_button) => Ok(stop_button),
		},
	}
}

async fn trigger_button(button: Arc<Box<dyn Button>>) -> Result<impl Reply, Rejection> {
	let button = button.clone();
	button.trigger().await;

	Ok(warp::redirect::see_other(Uri::from_static("/")))
}

pub async fn listen(garage: Garage, port: u16) {
	let index = warp::path::end()
		.and(warp::get())
		.and(with_garage(garage.clone()))
		.and_then(self::index);

	let door_toggle = warp::path!("door" / usize / "toggle")
		.and(warp::post())
		.and(with_garage(garage.clone()))
		.and_then(self::lookup_door)
		.and_then(self::extract_toggle_button)
		.and_then(self::trigger_button);

	let door_open = warp::path!("door" / usize / "open")
		.and(warp::post())
		.and(with_garage(garage.clone()))
		.and_then(self::lookup_door)
		.and_then(self::extract_open_button)
		.and_then(self::trigger_button);

	let door_close = warp::path!("door" / usize / "close")
		.and(warp::post())
		.and(with_garage(garage.clone()))
		.and_then(self::lookup_door)
		.and_then(self::extract_close_button)
		.and_then(self::trigger_button);

	let door_stop = warp::path!("door" / usize / "stop")
		.and(warp::post())
		.and(with_garage(garage.clone()))
		.and_then(self::lookup_door)
		.and_then(self::extract_stop_button)
		.and_then(self::trigger_button);

	let routes = index
		.or(door_toggle)
		.or(door_open)
		.or(door_close)
		.or(door_stop)
		.with(warp::trace::request());

	let address = SocketAddr::new(IpAddr::V4(Ipv4Addr::UNSPECIFIED), port);
	warp::serve(routes).run(address).await
}
