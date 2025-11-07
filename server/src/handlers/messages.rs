//! Order message API handlers
//!
//! REST API endpoints for vendor-buyer communication within orders.

use actix_session::Session;
use actix_web::{get, post, web, HttpRequest, HttpResponse, Responder};
use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use validator::Validate;

use crate::db::DbPool;
use crate::middleware::csrf::validate_csrf_token;
use crate::models::message::{NewOrderMessage, OrderMessage, OrderMessageWithSender};
use crate::models::order::Order;
use crate::models::user::User;

/// Request body for sending a message
#[derive(Debug, Deserialize, Validate)]
pub struct SendMessageRequest {
    #[validate(length(min = 1, max = 2000, message = "Message must be between 1 and 2000 characters"))]
    pub message: String,
}

/// Response for message list
#[derive(Debug, Serialize)]
pub struct MessagesResponse {
    pub messages: Vec<OrderMessageWithSender>,
    pub count: usize,
}

/// Helper to get authenticated user ID from session
fn get_user_id_from_session(session: &Session) -> Result<String, HttpResponse> {
    session
        .get::<String>("user_id")
        .map_err(|_| {
            HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "Failed to read session"
            }))
        })?
        .ok_or_else(|| {
            HttpResponse::Unauthorized().json(serde_json::json!({
                "error": "Not authenticated"
            }))
        })
}

/// GET /api/orders/{order_id}/messages - Get all messages for an order
///
/// Returns all messages for the specified order.
/// Only buyer or vendor can access messages for their own orders.
///
/// Requires authentication.
#[get("/orders/{order_id}/messages")]
pub async fn get_messages(
    pool: web::Data<DbPool>,
    session: Session,
    path: web::Path<String>,
) -> impl Responder {
    let order_id = path.into_inner();

    // Get authenticated user
    let user_id = match get_user_id_from_session(&session) {
        Ok(id) => id,
        Err(response) => return response,
    };

    // Get database connection
    let mut conn = match pool.get() {
        Ok(conn) => conn,
        Err(e) => {
            tracing::error!("Database connection error: {:?}", e);
            return HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "Database connection failed"
            }));
        }
    };

    // SECURITY: Verify user is buyer or vendor for this order
    let order = match Order::get_by_id(&order_id, &mut conn) {
        Ok(order) => order,
        Err(_) => {
            return HttpResponse::NotFound().json(serde_json::json!({
                "error": "Order not found"
            }));
        }
    };

    if order.buyer_id != user_id && order.vendor_id != user_id {
        return HttpResponse::Forbidden().json(serde_json::json!({
            "error": "You don't have permission to view these messages"
        }));
    }

    // Get messages
    let messages = match OrderMessage::get_by_order_id(&order_id, &mut conn) {
        Ok(msgs) => msgs,
        Err(e) => {
            tracing::error!("Failed to load messages: {:?}", e);
            return HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "Failed to load messages"
            }));
        }
    };

    // Enrich messages with sender info
    let messages_with_sender: Vec<OrderMessageWithSender> = messages
        .into_iter()
        .filter_map(|msg| {
            // Get sender username
            let sender = User::get_by_id(&msg.sender_id, &mut conn).ok()?;

            Some(OrderMessageWithSender {
                id: msg.id.clone(),
                order_id: msg.order_id.clone(),
                sender_id: msg.sender_id.clone(),
                sender_username: sender.username.clone(),
                message: msg.message.clone(),
                created_at: msg.created_at,
                is_current_user: msg.sender_id == user_id,
            })
        })
        .collect();

    HttpResponse::Ok().json(MessagesResponse {
        count: messages_with_sender.len(),
        messages: messages_with_sender,
    })
}

/// POST /api/orders/{order_id}/messages - Send a new message
///
/// Sends a new message in the order chat.
/// Only buyer or vendor can send messages for their own orders.
///
/// Requires authentication and CSRF protection.
#[post("/orders/{order_id}/messages")]
pub async fn send_message(
    pool: web::Data<DbPool>,
    session: Session,
    http_req: HttpRequest,
    path: web::Path<String>,
    req: web::Json<SendMessageRequest>,
) -> impl Responder {
    let order_id = path.into_inner();

    // SECURITY: Validate CSRF token
    let csrf_token = http_req
        .headers()
        .get("X-CSRF-Token")
        .and_then(|h| h.to_str().ok())
        .unwrap_or("");

    if !validate_csrf_token(&session, csrf_token) {
        return HttpResponse::Forbidden().json(serde_json::json!({
            "error": "Invalid or missing CSRF token"
        }));
    }

    // Validate input
    if let Err(e) = req.validate() {
        return HttpResponse::BadRequest().json(serde_json::json!({
            "error": format!("Validation error: {}", e)
        }));
    }

    // Get authenticated user
    let user_id = match get_user_id_from_session(&session) {
        Ok(id) => id,
        Err(response) => return response,
    };

    // Get database connection
    let mut conn = match pool.get() {
        Ok(conn) => conn,
        Err(e) => {
            tracing::error!("Database connection error: {:?}", e);
            return HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "Database connection failed"
            }));
        }
    };

    // SECURITY: Verify user is buyer or vendor for this order
    let order = match Order::get_by_id(&order_id, &mut conn) {
        Ok(order) => order,
        Err(_) => {
            return HttpResponse::NotFound().json(serde_json::json!({
                "error": "Order not found"
            }));
        }
    };

    if order.buyer_id != user_id && order.vendor_id != user_id {
        return HttpResponse::Forbidden().json(serde_json::json!({
            "error": "You don't have permission to send messages for this order"
        }));
    }

    // Create new message
    let new_message = NewOrderMessage::new(
        order_id.clone(),
        user_id.clone(),
        req.message.clone(),
    );

    let message = match OrderMessage::create(new_message, &mut conn) {
        Ok(msg) => msg,
        Err(e) => {
            tracing::error!("Failed to create message: {:?}", e);
            return HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "Failed to send message"
            }));
        }
    };

    // Get sender info for response
    let sender = match User::get_by_id(&user_id, &mut conn) {
        Ok(user) => user,
        Err(e) => {
            tracing::error!("Failed to load sender: {:?}", e);
            return HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "Failed to load sender info"
            }));
        }
    };

    let message_with_sender = OrderMessageWithSender {
        id: message.id.clone(),
        order_id: message.order_id.clone(),
        sender_id: message.sender_id.clone(),
        sender_username: sender.username.clone(),
        message: message.message.clone(),
        created_at: message.created_at,
        is_current_user: true,
    };

    HttpResponse::Ok().json(message_with_sender)
}
