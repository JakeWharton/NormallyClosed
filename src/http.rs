use crate::Garage;
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
		html.push_str(&format!(
			r#"<form action="/door/{}" method="post">
<label>{} <input type="submit" value="Toggle"></label>
</form>"#,
			i, door.door.name
		));
	}
	html.push_str(
		r#"</body>
</html>"#,
	);

	Ok(warp::reply::html(html))
}

async fn trigger(id: usize, garage: Garage) -> Result<impl Reply, Rejection> {
	if id >= garage.doors.len() {
		return Err(warp::reject());
	}

	garage.doors[id].trigger().await;

	Ok(warp::redirect::see_other(Uri::from_static("/")))
}

pub async fn listen(garage: Garage, port: u16) {
	let index = warp::path::end()
		.and(warp::get())
		.and(with_garage(garage.clone()))
		.and_then(self::index);

	let door_trigger = warp::path!("door" / usize)
		.and(warp::post())
		.and(with_garage(garage))
		.and_then(self::trigger);

	let routes = index.or(door_trigger).with(warp::trace::request());

	let address = SocketAddr::new(IpAddr::V4(Ipv4Addr::UNSPECIFIED), port);
	warp::serve(routes).run(address).await
}
