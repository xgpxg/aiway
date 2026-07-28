use logging::log;

pub(crate) async fn auto_renew() {
    log::info!("[cert_renew] task start");
    if let Err(e) = crate::server::cert::auto_renew().await {
        log::error!("[cert_renew] {}", e);
    }
    log::info!("[cert_renew] task end");
}
