use crate::entities::{bus, seat};
use crate::proto::seat::{BusResponse, GetSeatsResponse, SeatListResponse, SeatResponse};

pub fn to_seat_response(s: &seat::Model, b: Option<&bus::Model>) -> SeatResponse {
    SeatResponse {
        seat_id: s.seat_id,
        bus_id: s.bus_id,
        price: s.price,
        name: s.name.clone(),
        bus: b.map(|b| BusResponse {
            bus_id: b.bus_id,
            name: b.name.clone(),
            license_plate: b.license_plate.clone(),
        }),
    }
}

pub fn to_get_seats_response(
    rows: &[(seat::Model, Option<bus::Model>)],
    total: u64,
    offset: u64,
    size: u64,
) -> GetSeatsResponse {
    let data = rows
        .iter()
        .map(|(s, b)| to_seat_response(s, b.as_ref()))
        .collect();

    GetSeatsResponse {
        message: String::new(),
        size: size as i32,
        offset: offset as i32,
        total: total as i64,
        data,
    }
}

pub fn to_seat_list_response(models: &[seat::Model], message: String) -> SeatListResponse {
    let data = models
        .iter()
        .map(|s| to_seat_response(s, None))
        .collect();

    SeatListResponse { message, data }
}
