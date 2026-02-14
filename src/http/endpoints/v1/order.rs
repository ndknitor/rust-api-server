use crate::controllers::order::OrderController;
use crate::services::OrderServiceFactory;
use actix_web::{Responder, web};
use std::sync::Arc;

/// Scoped: factory.create() is called per request, creating a new OrderService instance
pub async fn get_orders(
    factory: web::Data<Arc<dyn OrderServiceFactory>>,
    path: web::Path<String>,
) -> impl Responder {
    let service = factory.create();
    let user_id = path.into_inner();
    let orders = service.get_orders(&user_id).await;
    OrderController::from_orders(orders).to_http()
}
