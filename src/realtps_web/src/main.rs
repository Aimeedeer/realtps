#[macro_use]
extern crate rocket;

use realtps_common::{Chain, Db, JsonDb, all_chains};
use rocket_dyn_templates::Template;
use serde::{Deserialize, Serialize};


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
        dbg!(&chain);
        dbg!(db.load_tps(chain));
        if let Some(tps) = db.load_tps(chain).expect(&format!("No tps data for chain {}", &chain)) {
            dbg!(&tps);
            list.push(
                Row {
                    chain,
                    tps,
                }
            );
            dbg!(&list);
        }
    }

    dbg!(&list);
    let context = Context {
        rows: list,
    };

    Template::render("index", &context)
}

#[launch]
fn rocket() -> _ {
    rocket::build()
        .mount("/", routes![index])
        .attach(Template::fairing())
}
