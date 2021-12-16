#[macro_use]
extern crate rocket;

use realtps_common::{Chain, Db, JsonDb, all_chains};
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
    let mut list = Vec::new();
    let db = JsonDb;

    for chain in all_chains() {
        if let Some(tps) = db.load_tps(chain).expect(&format!("No tps data for chain {}", &chain)) {
            list.push(
                Row {
                    chain,
                    tps,
                }
            );
        }
    }

    let context = Context {
        rows: list,
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
