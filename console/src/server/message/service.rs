use crate::server::db::Pool;
use crate::server::db::models::message;
use crate::server::db::models::message::{Message, MessageBuilder, MessageReadStatus};
use crate::server::message::MessageCountRes;
use crate::server::message::request::MessageListReq;
use crate::server::message::response::MessageListRes;
use aiway_protocol::common::req::{IdReq, Pagination};
use aiway_protocol::common::res::{IntoPageRes, PageRes};
use rbs::value;

pub(crate) async fn count_unread() -> anyhow::Result<MessageCountRes> {
    message::count_unread(Pool::get()?)
        .await
        .map_err(|e| anyhow::anyhow!(e))
}

pub(crate) async fn list(req: MessageListReq) -> anyhow::Result<PageRes<MessageListRes>> {
    let tx = Pool::get()?;
    let page = message::list_page(tx, &req.to_rb_page(), &req).await?;
    let mut res = page.convert_to_page_res(|list| {
        list.into_iter()
            .map(|item| MessageListRes { inner: item })
            .collect::<Vec<_>>()
    });

    let unread_count = message::count_unread(tx)
        .await
        .map_err(|e| anyhow::anyhow!(e))?;

    res.ext("unread_count", unread_count);
    Ok(res)
}

pub(crate) async fn read(req: IdReq) -> anyhow::Result<()> {
    let update = MessageBuilder::default()
        .read_status(Some(MessageReadStatus::Read))
        .build()?;

    if req.id == -1 {
        Message::update_by_map(
            Pool::get()?,
            &update,
            value! {"read_status": MessageReadStatus::Unread},
        )
        .await?;
        return Ok(());
    }

    Message::update_by_map(Pool::get()?, &update, value! {"id": req.id}).await?;
    Ok(())
}

pub(crate) async fn delete(req: IdReq) -> anyhow::Result<()> {
    Message::delete_by_map(Pool::get()?, value! {"id": req.id}).await?;
    Ok(())
}
