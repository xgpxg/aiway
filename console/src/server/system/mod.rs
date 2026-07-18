use rocket::routes;

mod notify;
mod agent_entry;
pub fn routes() -> Vec<rocket::Route> {
    routes![
        notify::api::update,
        notify::api::get,
        agent_entry::api::update,
        agent_entry::api::get,
    ]
}
