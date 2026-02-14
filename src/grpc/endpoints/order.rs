use crate::controllers::order::OrderController;
use crate::proto::order::service_server::Service as GrpcOrderService;
use crate::proto::order::{GetOrdersRequest, GetOrdersResponse};
use crate::services::OrderServiceFactory;
use std::sync::Arc;
use tonic::{Request, Response, Status};

/// OrderEndpoint holds a factory for Scoped lifetime
pub struct OrderEndpoint<F: OrderServiceFactory> {
    order_service_factory: Arc<F>,
}

impl<F: OrderServiceFactory> OrderEndpoint<F> {
    pub fn new(order_service_factory: Arc<F>) -> Self {
        Self { order_service_factory }
    }
}

#[tonic::async_trait]
impl<F: OrderServiceFactory + 'static> GrpcOrderService for OrderEndpoint<F> {
    async fn get_orders(
        &self,
        request: Request<GetOrdersRequest>,
    ) -> Result<Response<GetOrdersResponse>, Status> {
        let service = self.order_service_factory.create();
        let user_id = &request.into_inner().user_id;
        let orders = service.get_orders(user_id).await;
        OrderController::from_orders(orders).to_grpc()
    }
}
