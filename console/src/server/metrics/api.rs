use protocol::common::res::Res;
use rocket::routes;

pub fn routes() -> Vec<rocket::Route> {
    routes![]
}

// 网关整体运行状态
async fn gateway_global_state() -> Res<()> {

    Res::success(())
}
