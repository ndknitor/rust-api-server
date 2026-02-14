pub mod auth {
    tonic::include_proto!("api.v1.auth");
}

pub mod user {
    tonic::include_proto!("api.v1.user");
}

pub mod order {
    tonic::include_proto!("api.v1.order");
}
