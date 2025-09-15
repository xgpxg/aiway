pub enum SseEvent {
    Data(String),
    Error(String),
    Done,
}

impl SseEvent {
   pub fn to_string(&self) -> String {
        match self {
            SseEvent::Data(data) => format!("data: {}\n\n", data),
            SseEvent::Error(error) => {
                format!("event: error\ndata: {}\n\n", error)
            }
            SseEvent::Done => "data: [DONE]\n\n".to_string(),
        }
    }
}