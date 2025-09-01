use crate::app::get_app;
use crate::raft::RaftRequest;
use logging::log;
use std::sync::LazyLock;
use tokio::sync::mpsc;

pub enum Event {
    RaftRequestEvent(RaftRequest),
}

impl Event {
    pub fn send(self) -> Result<(), mpsc::error::SendError<Event>> {
        EVENT_BUS.send(self)
    }
}

pub struct EventBus {
    sender: mpsc::UnboundedSender<Event>,
}

impl EventBus {
    pub fn new() -> Self {
        let (sender, receiver) = mpsc::unbounded_channel::<Event>();
        let handler = EventHandler::new(receiver);

        // 启动事件处理任务
        tokio::spawn(async move {
            handler.handle_events().await;
        });

        Self { sender }
    }

    pub fn send(&self, event: Event) -> Result<(), mpsc::error::SendError<Event>> {
        self.sender.send(event)
    }
}

static EVENT_BUS: LazyLock<EventBus> = LazyLock::new(|| EventBus::new());

pub struct EventHandler {
    receiver: mpsc::UnboundedReceiver<Event>,
}

impl EventHandler {
    pub fn new(receiver: mpsc::UnboundedReceiver<Event>) -> Self {
        Self { receiver }
    }

    pub async fn handle_events(mut self) {
        while let Some(event) = self.receiver.recv().await {
            self.process_event(event).await;
        }
    }

    async fn process_event(&self, event: Event) {
        match event {
            Event::RaftRequestEvent(req) => {
                // 处理 Raft 请求事件
                self.handle_raft_request(req).await;
            }
        }
    }

    async fn handle_raft_request(&self, req: RaftRequest) {
        // 实现 Raft 请求处理逻辑
        match req {
            RaftRequest::Set { .. } => {}
            RaftRequest::Delete { .. } => {}
            RaftRequest::SetConfig { entry } => {
                match get_app()
                    .config_app
                    .manager
                    .upsert_config(
                        &entry.namespace_id,
                        &entry.id,
                        &entry.content,
                        entry.description.clone(),
                    )
                    .await
                {
                    Ok(_) => {}
                    Err(e) => {
                        log::error!("Error processing SetConfig request: {}", e);
                    }
                };
            }
            RaftRequest::DeleteConfig { namespace_id, id } => {}
        }
    }
}
