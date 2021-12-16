#[macro_use]
extern crate rocket;

use rocket_dyn_templates::Template;
use serde::{Serialize, Deserialize};
use realtps_common::Chain;

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
        }]
    };
    Template::render("index", &contextf)
}

#[launch]
fn rocket() -> _ {
    rocket::build()
        .mount("/", routes![index])
        .attach(Template::fairing())
}
