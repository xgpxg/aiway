use rocket::routes;

mod notify;
pub fn routes() -> Vec<rocket::Route> {
    routes![notify::api::update, notify::api::get]
}
