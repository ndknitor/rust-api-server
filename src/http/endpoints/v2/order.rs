use crate::controllers::order::OrderController;
use crate::services::OrderServiceTransient;
use actix_web::{Responder, web};

/// Transient: create_fn() is called every time we need a service instance
/// Multiple calls within the same request = multiple instances
pub async fn get_orders(
    create_fn: web::Data<OrderServiceTransient>,
    path: web::Path<String>,
) -> impl Responder {
    let user_id = path.into_inner();

    // Transient: new instance for first operation
    let service1 = create_fn();
    let orders = service1.get_orders(&user_id).await;

    // Transient: could create another instance for different operation
    // let service2 = create_fn();  // Different instance than service1

    OrderController::from_orders(orders).to_http()
}
