use rocket::routes;

pub(crate) mod acme;
mod notify;

pub fn routes() -> Vec<rocket::Route> {
    routes![
        notify::api::update,
        notify::api::get,
        acme::api::update,
        acme::api::get,
    ]
}
