use serde::Serialize;
#[derive(Debug, Serialize)]
pub struct WsTicketResponse {
    pub ws_ticket: String,
}
