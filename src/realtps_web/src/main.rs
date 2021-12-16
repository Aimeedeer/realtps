#[macro_use]
extern crate rocket;

use rocket_dyn_templates::Template;
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Debug)]
struct Context {
    
}

#[get("/")]
fn index() -> Template {
    let context = Context {};
    Template::render("index", &context)
}

#[launch]
fn rocket() -> _ {
    rocket::build()
        .mount("/", routes![index])
        .attach(Template::fairing())
}
