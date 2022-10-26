#[macro_use]
extern crate rocket;

use chrono::Duration;
use realtps_common::{
    chain::Chain,
    db::{CalculationLog, Db, JsonDb},
};
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
    chain_id: Chain,
    chain_name: String,
    note: Option<String>,
    tps: f64,
    tps_str: String,
    is_data_too_old: bool,
}

#[derive(Serialize, Deserialize, Debug)]
struct LogContext {
    log_list: Vec<Log>,
}

#[derive(Serialize, Deserialize, Debug)]
struct Log {
    chain_id: Chain,
    chain_name: String,
    log_details: CalculationLog,
}

#[get("/")]
fn index() -> Template {
    let mut list = Vec::new();
    let db = JsonDb;

    for chain in Chain::all_chains() {
        if let Some(tps) = db
            .load_tps(chain)
            .unwrap_or_else(|_| panic!("No tps data for chain {}", &chain))
        {
            let mut is_data_too_old = false;
            if let Some(log_details) = db
                .load_calculation_log(chain)
                .unwrap_or_else(|_| panic!("No calculation log for chain {}", &chain))
            {
                if log_details.calculating_start - log_details.newest_block_timestamp
                    > Duration::days(1)
                {
                    is_data_too_old = true;
                }
            }

            let note = chain_note(chain).map(ToString::to_string);
            let chain_id = chain;
            let chain_name = chain.description().to_string();
            let tps_str = format!("{:.2}", tps);

            list.push(Row {
                chain_id,
                chain_name,
                note,
                tps,
                tps_str,
                is_data_too_old,
            });
        }
    }

    let context = Context { rows: list };
    Template::render("index", &context)
}

#[get("/log")]
fn log() -> Template {
    let mut list = Vec::new();
    let db = JsonDb;

    for chain in Chain::all_chains() {
        if let Some(log_details) = db
            .load_calculation_log(chain)
            .unwrap_or_else(|_| panic!("No calculation log for chain {}", &chain))
        {
            let chain_id = chain;
            let chain_name = chain.description().to_string();

            list.push(Log {
                chain_id,
                chain_name,
                log_details,
            });
        }
    }

    let context = LogContext { log_list: list };
    Template::render("log", &context)
}

#[get("/about")]
fn about() -> Template {
    Template::render("about", EmptyContext {})
}

#[launch]
fn rocket() -> _ {
    rocket::build()
        .mount("/", routes![index, about, log])
        .mount("/static", FileServer::from(relative!("static")))
        .attach(Template::fairing())
}

fn chain_note(chain: Chain) -> Option<&'static str> {
    match chain {
        Chain::Solana => Some("solana"),
        _ => None,
    }
}
