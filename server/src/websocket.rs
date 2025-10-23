//! WebSocket server for real-time notifications

use actix::{Actor, ActorContext, Addr, AsyncContext, Context, Handler, Message, StreamHandler};
use actix_web_actors::ws;
use anyhow::Result;
use serde;
use std::collections::{HashMap, HashSet};
use std::time::{Duration, Instant};
use tracing::{info, warn};
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
            _ => {}
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
    EscrowInit {
        escrow_id: Uuid,
    },
    EscrowAssigned {
        escrow_id: Uuid,
    },
    EscrowStatusChanged {
        escrow_id: Uuid,
        new_status: String,
    },
    TransactionConfirmed {
        tx_hash: String,
        confirmations: u32,
    },
    NewMessage {
        from: Uuid,
        content: String,
    },
    OrderStatusChanged {
        order_id: Uuid,
        new_status: String,
    },
    DisputeResolved {
        escrow_id: Uuid,
        resolution: String,
        decided_by: Uuid,
    },
    /// Invitation to submit a review after escrow transaction completion
    ///
    /// Triggered automatically when a transaction is confirmed on the blockchain.
    /// The buyer receives this notification to invite them to rate the vendor.
    ReviewInvitation {
        escrow_id: Uuid,
        tx_hash: String,
        buyer_id: Uuid,
        vendor_id: Uuid,
    },
}

// --- Handlers ---

impl Handler<Connect> for WebSocketServer {
    type Result = ();

    fn handle(&mut self, msg: Connect, _: &mut Context<Self>) {
        info!(
            "WebSocket session {} connected for user {}",
            msg.id, msg.user_id
        );
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
        // Serialize event to JSON
        let json_msg = match serde_json::to_string(&msg) {
            Ok(json) => json,
            Err(e) => {
                warn!("Failed to serialize WebSocket event: {}", e);
                return;
            }
        };

        // Broadcast to all connected sessions
        // In a production system, we would filter by user_id based on the event type
        for addr in self.sessions.values() {
            addr.do_send(WsMessage(json_msg.clone()));
        }

        info!("Broadcast WebSocket event: {:?}", msg);
    }
}

/// Message to notify a specific user
#[derive(Message)]
#[rtype(result = "()")]
pub struct NotifyUser {
    pub user_id: Uuid,
    pub event: WsEvent,
}

impl Handler<NotifyUser> for WebSocketServer {
    type Result = ();

    fn handle(&mut self, msg: NotifyUser, _ctx: &mut Context<Self>) {
        // Find all sessions for this user
        let session_ids = match self.user_sessions.get(&msg.user_id) {
            Some(ids) => ids,
            None => {
                info!(
                    "User {} has no active WebSocket sessions, cannot notify",
                    msg.user_id
                );
                return;
            }
        };

        // Serialize event to JSON
        let json_msg = match serde_json::to_string(&msg.event) {
            Ok(json) => json,
            Err(e) => {
                warn!("Failed to serialize WebSocket event: {}", e);
                return;
            }
        };

        // Send to all sessions for this user
        let mut notified_count = 0;
        for session_id in session_ids {
            if let Some(addr) = self.sessions.get(session_id) {
                addr.do_send(WsMessage(json_msg.clone()));
                notified_count += 1;
            }
        }

        info!(
            "Notified user {} via {} WebSocket session(s): {:?}",
            msg.user_id, notified_count, msg.event
        );
    }
}
