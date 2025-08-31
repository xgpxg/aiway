mod app;
mod manage;
mod raft;

pub fn routes() -> Vec<rocket::Route> {
    routes![
        raft::vote,
        manage::init,
        manage::metrics,
        manage::change_membership,
        manage::add_learner,
        app::read,
        app::write,
    ]
}
