#[macro_use]
extern crate rocket;

use realtps_common::Chain;
use rocket_dyn_templates::Template;
use serde::{Deserialize, Serialize};
use rocket::fs::{FileServer, relative};

#[derive(Serialize, Deserialize, Debug)]
struct Context {
    rows: Vec<Row>,
}

#[derive(Serialize, Deserialize, Debug)]
struct Row {
    chain: Chain,
    tps: f64,
}

#[get("/")]
fn index() -> Template {
    // test
    let context = Context {
        rows: vec![Row {
            chain: Chain::Polygon,
            tps: 32.98,
        }],
    };
    Template::render("index", &context)
}

#[launch]
fn rocket() -> _ {
    rocket::build()
        .mount("/", routes![index])
        .mount("/static", FileServer::from(relative!("static")))
        .attach(Template::fairing())
}
