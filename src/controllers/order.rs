use crate::proto::order::{GetOrdersResponse, Order as ProtoOrder};
use crate::services::Order;
use actix_web::HttpResponse;
use tonic::Response;

pub struct OrderController(pub GetOrdersResponse);

impl OrderController {
    pub fn from_orders(orders: Vec<Order>) -> Self {
        let proto_orders: Vec<ProtoOrder> = orders
            .into_iter()
            .map(|o| ProtoOrder {
                id: o.id,
                user_id: o.user_id,
                product: o.product,
                quantity: o.quantity,
            })
            .collect();
        Self(GetOrdersResponse { orders: proto_orders })
    }

    pub fn to_http(&self) -> HttpResponse {
        HttpResponse::Ok().json(&self.0)
    }

    pub fn to_grpc(self) -> Result<Response<GetOrdersResponse>, tonic::Status> {
        Ok(Response::new(self.0))
    }
}
