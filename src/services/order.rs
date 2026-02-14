use async_trait::async_trait;

pub struct Order {
    pub id: String,
    pub user_id: String,
    pub product: String,
    pub quantity: i32,
}

#[async_trait]
pub trait OrderService: Send + Sync {
    async fn get_orders(&self, user_id: &str) -> Vec<Order>;
}

pub struct OrderServiceImpl;

#[async_trait]
impl OrderService for OrderServiceImpl {
    async fn get_orders(&self, user_id: &str) -> Vec<Order> {
        vec![
            Order {
                id: "1".to_string(),
                user_id: user_id.to_string(),
                product: "Laptop".to_string(),
                quantity: 1,
            },
            Order {
                id: "2".to_string(),
                user_id: user_id.to_string(),
                product: "Mouse".to_string(),
                quantity: 2,
            },
        ]
    }
}

// ============================================================================
// SCOPED: Factory creates one instance per request
// ============================================================================
pub trait OrderServiceFactory: Send + Sync {
    fn create(&self) -> Box<dyn OrderService>;
}

pub struct OrderServiceFactoryImpl;

impl OrderServiceFactory for OrderServiceFactoryImpl {
    fn create(&self) -> Box<dyn OrderService> {
        Box::new(OrderServiceImpl)
    }
}

// ============================================================================
// TRANSIENT: Function type creates new instance every call
// ============================================================================
pub type OrderServiceTransient = fn() -> Box<dyn OrderService>;

pub fn create_order_service() -> Box<dyn OrderService> {
    Box::new(OrderServiceImpl)
}
