use crate::garage::Button;
use crate::garage::DoorControl;
use crate::garage::Garage;
use crate::sync;
use itertools::Itertools;
use std::convert::Infallible;
use std::net::IpAddr;
use std::net::Ipv4Addr;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::sync::Mutex;
use warp::http::Uri;
use warp::Filter;
use warp::Rejection;
use warp::Reply;

async fn index(garage: Arc<Mutex<Garage>>) -> Result<impl Reply, Infallible> {
	let garage = garage.lock().await;

	let mut html = r#"<!DOCTYPE html>
<html lang="en">
<head>
<meta charset="utf-8">
<meta name="viewport" content="width=device-width,initial-scale=1">
<title>Normally Closed</title>
<style type="text/css">
body {
	background-color: #415A8D;
	color: #fff;
	font-family: Arial, Helvetica, sans-serif;
}
h1 small {
	font-size: 0.5em;
	font-weight: normal;
	opacity: 0.5;
}
form {
	display: inline-block;
}
input {
	font-size: 1.2em;
	padding: 1em;
}
footer {
	margin-top: 50px;
	font-size: .8em;
}
a, a:visited, a:hover, a:active {
	color: #fff;
	text-decoration: underline dotted;
}
a:hover {
	text-decoration: underline;
}
</style>
</head>
<body>
"#
	.to_string();

	for (i, door) in garage
		.doors
		.iter()
		.sorted_by_key(|door| &door.name)
		.enumerate()
	{
		html.push_str("<h1>");
		html.push_str(&door.name);
		if let Some(host) = &door.host {
			html.push_str(r#" <small>via <a href="http://"#);
			html.push_str(host);
			html.push_str(r#"">"#);
			html.push_str(host);
			html.push_str("</a></small>");
		}
		html.push_str("</h1>\n");
		match &door.control {
			DoorControl::Toggle { .. } => {
				html.push_str(&format!(
					r#"<form action="/door/{}/toggle" method="post">
<input type="submit" value="Toggle">
</form>
"#,
					i
				));
			}
			DoorControl::Discrete {
				open_button: _,
				close_button: _,
				stop_button,
			} => {
				html.push_str(&format!(
					r#"<form action="/door/{}/open" method="post">
<input type="submit" value="Open">
</form>
<form action="/door/{}/close" method="post">
<input type="submit" value="Close">
</form>
"#,
					i, i
				));
				if stop_button.is_some() {
					html.push_str(&format!(
						r#"<form action="/door/{}/stop" method="post">
<input type="submit" value="Stop">
</form>
"#,
						i
					));
				}
			}
		};
	}
	html.push_str(
		r#"<footer>Powered by <a href="https://github.com/JakeWharton/NormallyClosed">NormallyClosed</a>.</footer>
</body>
</html>"#,
	);

	Ok(warp::reply::html(html))
}

async fn doors_json(garage: Arc<Mutex<Garage>>) -> Result<impl Reply, Infallible> {
	let garage = garage.lock().await;

	let json = sync::garage_json(&garage);
	Ok(warp::reply::json(&json))
}

async fn lookup_door(id: usize, garage: &Garage) -> Result<&DoorControl, Rejection> {
	if id >= garage.doors.len() {
		return Err(warp::reject());
	}
	let door = &garage.doors[id];
	Ok(&door.control)
}

async fn toggle(id: usize, garage: Arc<Mutex<Garage>>) -> Result<impl Reply, Rejection> {
	let garage = garage.lock().await;
	let door = lookup_door(id, &garage).await?;
	if let DoorControl::Toggle { button } = door {
		trigger_button(&button).await
	} else {
		Err(warp::reject())
	}
}

async fn open(id: usize, garage: Arc<Mutex<Garage>>) -> Result<impl Reply, Rejection> {
	let garage = garage.lock().await;
	let door = lookup_door(id, &garage).await?;
	if let DoorControl::Discrete {
		open_button,
		close_button: _,
		stop_button: _,
	} = door
	{
		trigger_button(&open_button).await
	} else {
		Err(warp::reject())
	}
}

async fn close(id: usize, garage: Arc<Mutex<Garage>>) -> Result<impl Reply, Rejection> {
	let garage = garage.lock().await;
	let door = lookup_door(id, &garage).await?;
	if let DoorControl::Discrete {
		open_button: _,
		close_button,
		stop_button: _,
	} = door
	{
		trigger_button(&close_button).await
	} else {
		Err(warp::reject())
	}
}

async fn stop(id: usize, garage: Arc<Mutex<Garage>>) -> Result<impl Reply, Rejection> {
	let garage = garage.lock().await;
	let door = lookup_door(id, &garage).await?;
	if let DoorControl::Discrete {
		open_button: _,
		close_button: _,
		stop_button,
	} = door
	{
		if let Some(stop_button) = stop_button {
			trigger_button(&stop_button).await
		} else {
			Err(warp::reject())
		}
	} else {
		Err(warp::reject())
	}
}

async fn trigger_button(button: &Box<dyn Button>) -> Result<impl Reply, Rejection> {
	button.trigger().await;

	Ok(warp::redirect::see_other(Uri::from_static("/")))
}

pub async fn listen(garage: Arc<Mutex<Garage>>, port: u16) {
	let garage = warp::any().map(move || garage.clone());

	let index = warp::path::end()
		.and(warp::get())
		.and(garage.clone())
		.and_then(self::index);

	let doors_json = warp::path!("doors.json")
		.and(warp::get())
		.and(garage.clone())
		.and_then(self::doors_json);

	let door_toggle = warp::path!("door" / usize / "toggle")
		.and(warp::post())
		.and(garage.clone())
		.and_then(self::toggle);

	let door_open = warp::path!("door" / usize / "open")
		.and(warp::post())
		.and(garage.clone())
		.and_then(self::open);

	let door_close = warp::path!("door" / usize / "close")
		.and(warp::post())
		.and(garage.clone())
		.and_then(self::close);

	let door_stop = warp::path!("door" / usize / "stop")
		.and(warp::post())
		.and(garage.clone())
		.and_then(self::stop);

	let routes = index
		.or(doors_json)
		.or(door_toggle)
		.or(door_open)
		.or(door_close)
		.or(door_stop)
		.with(warp::trace::request());

	let address = SocketAddr::new(IpAddr::V4(Ipv4Addr::UNSPECIFIED), port);
	println!("HTTP listening at http://{}", address);

	warp::serve(routes).run(address).await
}
