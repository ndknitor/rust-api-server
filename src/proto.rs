pub mod common {
    tonic::include_proto!("api.v1.common");
}

pub mod auth {
    tonic::include_proto!("api.v1.auth");
}

pub mod seat {
    tonic::include_proto!("api.v1.seat");
}
