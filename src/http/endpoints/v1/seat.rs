use crate::controllers::seat as seat_ctrl;
use crate::inject::InjectFactory;
use crate::proto::seat::{
    CreateSeatInput as ProtoCreateSeatInput, UpdateSeatInput as ProtoUpdateSeatInput,
};
use crate::services::seat::{CreateSeatInput, SeatError, UpdateSeatInput};
use actix_web::{web, HttpResponse, Responder};
use serde::Deserialize;
use std::sync::Arc;

fn to_http_error(e: SeatError) -> HttpResponse {
    match e {
        SeatError::BusNotFound(id) => {
            HttpResponse::BadRequest().json(format!("Bus with id {} does not exist", id))
        }
        SeatError::SeatNotFound(id) => {
            HttpResponse::NotFound().json(format!("Seat with id {} does not exist", id))
        }
        SeatError::Db(db_err) => HttpResponse::InternalServerError().json(db_err.to_string()),
    }
}

#[derive(Debug, Deserialize)]
pub struct GetSeatsQuery {
    pub offset: Option<u64>,
    pub size: Option<u64>,
}

#[derive(Debug, Deserialize)]
pub struct GetSeatsByRangeQuery {
    pub seat_id: Option<String>,
    pub name: Option<String>,
}

/// GET /api/v1/seats?offset=0&size=10
pub async fn get_seats(
    factory: web::Data<Arc<dyn InjectFactory>>,
    query: web::Query<GetSeatsQuery>,
) -> impl Responder {
    let offset = query.offset.unwrap_or(0);
    let size = query.size.unwrap_or(10);

    let svc = factory.seat_service();
    match svc.get_seats(offset, size).await {
        Ok((rows, total)) => {
            HttpResponse::Ok().json(seat_ctrl::to_get_seats_response(&rows, total, offset, size))
        }
        Err(e) => HttpResponse::InternalServerError().json(e.to_string()),
    }
}

/// GET /api/v1/seats/range?seat_id=1,2,3&name=A1,A2
pub async fn get_seats_by_range(
    factory: web::Data<Arc<dyn InjectFactory>>,
    query: web::Query<GetSeatsByRangeQuery>,
) -> impl Responder {
    let seat_ids: Vec<i32> = query
        .seat_id
        .as_deref()
        .unwrap_or("")
        .split(',')
        .filter_map(|s| s.trim().parse().ok())
        .collect();

    let names: Vec<String> = query
        .name
        .as_deref()
        .unwrap_or("")
        .split(',')
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect();

    if seat_ids.is_empty() && names.is_empty() {
        return HttpResponse::Ok().json(seat_ctrl::to_get_seats_response(&[], 0, 0, 0));
    }

    let svc = factory.seat_service();
    match svc.get_seats_by_range(seat_ids, names).await {
        Ok(rows) => {
            let total = rows.len() as u64;
            HttpResponse::Ok().json(seat_ctrl::to_get_seats_response(&rows, total, 0, total))
        }
        Err(e) => HttpResponse::InternalServerError().json(e.to_string()),
    }
}

/// POST /api/v1/seats
pub async fn create_seats(
    factory: web::Data<Arc<dyn InjectFactory>>,
    payload: web::Json<Vec<ProtoCreateSeatInput>>,
) -> impl Responder {
    let proto_inputs = payload.into_inner();
    if proto_inputs.is_empty() {
        return HttpResponse::Ok().json(seat_ctrl::to_seat_list_response(
            &[],
            "No seats to create".into(),
        ));
    }

    let inputs: Vec<CreateSeatInput> = proto_inputs
        .into_iter()
        .map(|s| CreateSeatInput {
            bus_id: s.bus_id,
            price: s.price,
            name: s.name,
        })
        .collect();

    let svc = factory.seat_service();
    match svc.create_seats(&inputs).await {
        Ok(created) => {
            let msg = format!("Created {} seat(s)", created.len());
            HttpResponse::Ok().json(seat_ctrl::to_seat_list_response(&created, msg))
        }
        Err(e) => to_http_error(e),
    }
}

/// PUT /api/v1/seats
pub async fn update_seats(
    factory: web::Data<Arc<dyn InjectFactory>>,
    payload: web::Json<Vec<ProtoUpdateSeatInput>>,
) -> impl Responder {
    let proto_inputs = payload.into_inner();
    if proto_inputs.is_empty() {
        return HttpResponse::Ok().json(seat_ctrl::to_seat_list_response(
            &[],
            "No seats to update".into(),
        ));
    }

    let inputs: Vec<UpdateSeatInput> = proto_inputs
        .into_iter()
        .map(|s| UpdateSeatInput {
            seat_id: s.seat_id,
            bus_id: s.bus_id,
            price: s.price,
            name: s.name,
        })
        .collect();

    let svc = factory.seat_service();
    match svc.update_seats(&inputs).await {
        Ok(updated) => {
            let msg = format!("Updated {} seat(s)", updated.len());
            HttpResponse::Ok().json(seat_ctrl::to_seat_list_response(&updated, msg))
        }
        Err(e) => to_http_error(e),
    }
}

/// DELETE /api/v1/seats
pub async fn delete_seats(
    factory: web::Data<Arc<dyn InjectFactory>>,
    payload: web::Json<Vec<i32>>,
) -> impl Responder {
    let seat_ids = payload.into_inner();
    if seat_ids.is_empty() {
        return HttpResponse::Ok().json(seat_ctrl::to_seat_list_response(
            &[],
            "No seats to delete".into(),
        ));
    }

    let svc = factory.seat_service();
    match svc.delete_seats(seat_ids).await {
        Ok(rows_affected) => HttpResponse::Ok().json(seat_ctrl::to_seat_list_response(
            &[],
            format!("Deleted {} seat(s)", rows_affected),
        )),
        Err(e) => HttpResponse::InternalServerError().json(e.to_string()),
    }
}
