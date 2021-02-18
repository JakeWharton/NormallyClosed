use crate::Garage;
use std::io;
use tide::http::mime;
use tide::Redirect;
use tide::Response;
use tide::StatusCode::BadRequest;
use tide_tracing::TraceMiddleware;

pub async fn listen(garage: Garage) -> io::Result<()> {
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

	app.listen("0.0.0.0:8080").await
}
