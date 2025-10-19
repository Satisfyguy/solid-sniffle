//! WebSocket server for real-time notifications

use actix::{
    Actor,
    ActorContext,
    Addr,
    AsyncContext,
    Context,
    Handler,
    Message,
    StreamHandler,
};
use actix_web_actors::ws;
use anyhow::Result;
use std::collections::{HashMap, HashSet};
use std::time::{Duration, Instant};
use tracing::{error, info, warn};
use serde;
use uuid::Uuid;

const HEARTBEAT_INTERVAL: Duration = Duration::from_secs(5);
const CLIENT_TIMEOUT: Duration = Duration::from_secs(10);

// --- WebSocket Session Actor ---

pub struct WebSocketSession {
    pub id: Uuid,
    pub user_id: Uuid,
    pub hb: Instant,
    pub server: Addr<WebSocketServer>,
}

impl Actor for WebSocketSession {
    type Context = ws::WebsocketContext<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        self.hb(ctx);
        self.server.do_send(Connect {
            id: self.id,
            user_id: self.user_id,
            addr: ctx.address(),
        });
    }

    fn stopped(&mut self, _ctx: &mut Self::Context) {
        self.server.do_send(Disconnect { id: self.id });
    }
}

impl WebSocketSession {
    fn hb(&self, ctx: &mut ws::WebsocketContext<Self>) {
        ctx.run_interval(HEARTBEAT_INTERVAL, |act, ctx| {
            if Instant::now().duration_since(act.hb) > CLIENT_TIMEOUT {
                warn!("Heartbeat timeout, disconnecting session {}", act.id);
                ctx.stop();
                return;
            }
            ctx.ping(b"");
        });
    }
}

impl StreamHandler<Result<ws::Message, ws::ProtocolError>> for WebSocketSession {
    fn handle(&mut self, msg: Result<ws::Message, ws::ProtocolError>, ctx: &mut Self::Context) {
        match msg {
            Ok(ws::Message::Ping(msg)) => {
                self.hb = Instant::now();
                ctx.pong(&msg);
            }
            Ok(ws::Message::Pong(_)) => {
                self.hb = Instant::now();
            }
            Ok(ws::Message::Close(reason)) => {
                ctx.close(reason);
                ctx.stop();
            }
            _ => {},
        }
    }
}

impl Handler<WsMessage> for WebSocketSession {
    type Result = ();

    fn handle(&mut self, msg: WsMessage, ctx: &mut <Self as Actor>::Context) {
        ctx.text(msg.0);
    }
}
// --- WebSocket Server Actor ---

#[derive(Default)]
pub struct WebSocketServer {
    sessions: HashMap<Uuid, Addr<WebSocketSession>>,
    user_sessions: HashMap<Uuid, HashSet<Uuid>>,
}

impl Actor for WebSocketServer {
    type Context = Context<Self>;

    fn started(&mut self, _ctx: &mut Context<Self>) {
        info!("WebSocketServer actor started");
    }
}

// --- Messages ---

#[derive(Message)]
#[rtype(result = "()")]
pub struct Connect {
    pub id: Uuid,
    pub user_id: Uuid,
    pub addr: Addr<WebSocketSession>,
}

#[derive(Message)]
#[rtype(result = "()")]
pub struct Disconnect {
    pub id: Uuid,
}

#[derive(Message, Clone)]
#[rtype(result = "()")]
pub struct WsMessage(pub String);

#[derive(Message, Debug, Clone, serde::Serialize)]
#[rtype(result = "()")]
pub enum WsEvent {
    EscrowInit { escrow_id: Uuid },
    EscrowAssigned { escrow_id: Uuid },
    EscrowStatusChanged { escrow_id: Uuid, new_status: String },
    TransactionConfirmed { tx_hash: String, confirmations: u32 },
    NewMessage { from: Uuid, content: String },
    OrderStatusChanged { order_id: Uuid, new_status: String },
}

// --- Handlers ---

impl Handler<Connect> for WebSocketServer {
    type Result = ();

    fn handle(&mut self, msg: Connect, _: &mut Context<Self>) {
        info!("WebSocket session {} connected for user {}", msg.id, msg.user_id);
        self.sessions.insert(msg.id, msg.addr);
        self.user_sessions
            .entry(msg.user_id)
            .or_default()
            .insert(msg.id);
    }
}

impl Handler<Disconnect> for WebSocketServer {
    type Result = ();

    fn handle(&mut self, msg: Disconnect, _: &mut Context<Self>) {
        info!("WebSocket session {} disconnected", msg.id);
        if self.sessions.remove(&msg.id).is_some() {
            // This is inefficient, but acceptable for now.
            // A better implementation would use a reverse map.
            for sessions in self.user_sessions.values_mut() {
                sessions.remove(&msg.id);
            }
        }
    }
}

impl Handler<WsEvent> for WebSocketServer {
    type Result = ();

    fn handle(&mut self, msg: WsEvent, _ctx: &mut Context<Self>) {
        let user_id = match &msg {
            WsEvent::EscrowInit { escrow_id } => {
                // This is a simplification. We would need to look up the user_id from the escrow_id.
                warn!("Cannot notify for EscrowInit without user_id");
                return;
            }
            _ => {
                // This is a simplification. We would need to look up the user_id from the event.
                warn!("Cannot notify for this event without user_id");
                return;
            }
        };

        if let Some(sessions) = self.user_sessions.get(&user_id) {
            let payload = match serde_json::to_string(&msg) {
                Ok(p) => p,
                Err(e) => {
                    error!("Failed to serialize WebSocket message: {}", e);
                    return;
                }
            };
            for session_id in sessions {
                if let Some(addr) = self.sessions.get(session_id) {
                    addr.do_send(WsMessage(payload.clone()));
                }
            }
        }
    }
}

impl WebSocketServer {
    pub async fn notify(&self, user_id: String, event: WsEvent) -> Result<()> {
        // This is a placeholder. The actual notification logic is in the Handler<WsEvent>.
        // This function can be removed once the EscrowOrchestrator is updated to send messages to the actor.
        info!("NOTIFY {}: {:?}", user_id, event);
        Ok(())
    }
}