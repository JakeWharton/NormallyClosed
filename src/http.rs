use crate::Garage;
use async_std::net::IpAddr;
use async_std::net::Ipv4Addr;
use async_std::net::SocketAddr;
use std::io;
use tide::http::mime;
use tide::Redirect;
use tide::Response;
use tide::StatusCode::BadRequest;
use tide_tracing::TraceMiddleware;

pub async fn listen(garage: Garage, port: u16) -> io::Result<()> {
	let mut app = tide::with_state(garage);
	app.with(TraceMiddleware::new());

	app.at("/").get(|req: tide::Request<Garage>| async move {
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
		for (i, door) in state.doors.iter().enumerate() {
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

		Ok(Response::builder(200).body(html).content_type(mime::HTML))
	});

	app
		.at("/door/:id")
		.post(|req: tide::Request<Garage>| async move {
			let state = req.state();

			let id: usize = req.param("id")?.parse()?;
			if id >= state.doors.len() {
				return Ok(Response::new(BadRequest));
			}

			state.doors[id].trigger().await;

			Ok(Redirect::new("/").into())
		});

	let address = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0)), port);
	app.listen(address).await
}
