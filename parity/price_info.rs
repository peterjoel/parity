use rustc_serialize::json::Json;
use std::io::Read;
use hyper::Client;
use hyper::header::Connection;
use std::str::FromStr;

pub struct PriceInfo {
	pub ethusd: f32,
}

impl PriceInfo {
	pub fn get() -> Option<PriceInfo> {
		let mut body = String::new();
		// TODO: Handle each error type properly
		Client::new()
			.get("http://api.etherscan.io/api?module=stats&action=ethprice")
			.header(Connection::close())
			.send().ok()
			.and_then(|mut s| s.read_to_string(&mut body).ok())
			.and_then(|_| Json::from_str(&body).ok())
			.and_then(|json| {
				let ethusd: f32 = if let Some(&Json::String(ref s)) = json.find_path(&["result", "ethusd"]) {
					FromStr::from_str(&s).unwrap()
				} else {
					return None;
				};
				Some(PriceInfo {
					ethusd: ethusd,
				})
			})
	}
}
