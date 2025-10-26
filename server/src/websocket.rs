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
    /// Warning that an escrow is approaching expiration
    ///
    /// Triggered when an escrow is within the warning threshold (default 1h) of its deadline.
    /// Parties should complete required actions or the escrow will be auto-cancelled/refunded.
    EscrowExpiring {
        escrow_id: Uuid,
        status: String,
        expires_in_secs: u64,
        action_required: String,
    },
    /// Notification that an escrow has expired
    ///
    /// Triggered when an escrow exceeds its timeout for the current status.
    /// The escrow status has been updated to "expired" or "cancelled".
    EscrowExpired {
        escrow_id: Uuid,
        previous_status: String,
        reason: String,
    },
    /// Notification that an escrow was automatically cancelled due to timeout
    ///
    /// Occurs when setup/funding takes too long. No funds were lost as
    /// multisig was not funded or transaction did not complete.
    EscrowAutoCancelled {
        escrow_id: Uuid,
        reason: String,
        cancelled_at_status: String,
    },
    /// Notification that a dispute has been escalated due to timeout
    ///
    /// Occurs when an arbiter does not resolve a dispute within the timeout period.
    /// Admin intervention is now required, or automatic refund has been triggered.
    DisputeEscalated {
        escrow_id: Uuid,
        arbiter_id: Uuid,
        days_in_dispute: u64,
        action_taken: String,
    },
    /// Alert that a transaction appears stuck (high confirmation timeout)
    ///
    /// Triggered when a "releasing" or "refunding" transaction has not confirmed
    /// within the expected timeframe. May indicate blockchain congestion or other issues.
    TransactionStuck {
        escrow_id: Uuid,
        tx_hash: String,
        hours_pending: u64,
        suggested_action: String,
    },
    /// Alert that multisig setup has stalled
    ///
    /// Triggered when an escrow in "created" status has had no progress for >15 minutes.
    /// May indicate wallet RPC connectivity issues or client disconnection.
    MultisigSetupStuck {
        escrow_id: Uuid,
        minutes_stuck: u64,
        last_step: String,
        suggested_action: String,
    },
    /// Alert that multisig setup has failed permanently
    ///
    /// Triggered when MultisigStateRepository marks an escrow as failed.
    /// Indicates unrecoverable error requiring manual intervention or escrow cancellation.
    MultisigSetupFailed {
        escrow_id: Uuid,
        reason: String,
        failed_at_step: String,
        can_retry: bool,
    },
    /// Notification that wallet recovery was successful
    ///
    /// Triggered after server restart when WalletManager successfully recovers
    /// an escrow's wallet state from persisted RPC configs and multisig snapshots.
    MultisigRecovered {
        escrow_id: Uuid,
        recovered_wallets: Vec<String>, // ["buyer", "vendor", "arbiter"]
        phase: String,
        recovered_at: i64, // Unix timestamp
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
