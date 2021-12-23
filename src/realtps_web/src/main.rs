#[macro_use]
extern crate rocket;

use realtps_common::{all_chains, chain_description, Db, JsonDb, Chain};
use rocket::fs::{relative, FileServer};
use rocket_dyn_templates::Template;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
struct EmptyContext {}

#[derive(Serialize, Deserialize, Debug)]
struct Context {
    rows: Vec<Row>,
}

#[derive(Serialize, Deserialize, Debug)]
struct Row {
    chain: String,
    note: Option<String>,
    tps: f64,
    tps_str: String,
}

#[get("/")]
fn index() -> Template {
    let mut list = Vec::new();
    let db = JsonDb;

    for chain in all_chains() {
        if let Some(tps) = db
            .load_tps(chain)
            .expect(&format!("No tps data for chain {}", &chain))
        {
            let note = chain_note(chain).map(ToString::to_string);
            let chain = chain_description(chain).to_string();
            let tps_str = format!("{:.2}", tps);
            list.push(Row {
                chain,
                note,
                tps,
                tps_str,
            });
        }
    }

    let context = Context { rows: list };
    Template::render("index", &context)
}

#[get("/about")]
fn about() -> Template {
    Template::render("about", EmptyContext {})
}

#[launch]
fn rocket() -> _ {
    rocket::build()
        .mount("/", routes![index, about])
        .mount("/static", FileServer::from(relative!("static")))
        .attach(Template::fairing())
}

fn chain_note(chain: Chain) -> Option<&'static str> {
    match chain {
        Chain::Solana => Some("solana"),
        _ => None,
    }
}
