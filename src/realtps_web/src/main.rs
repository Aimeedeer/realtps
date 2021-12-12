#[macro_use]
extern crate rocket;

#[get("/")]
fn index() -> &'static str {
    "Hello, Real TPS!"
}

#[launch]
fn rocket() -> _ {
    rocket::build().mount("/", routes![index])
}
