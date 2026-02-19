use crate::controllers::seat as seat_ctrl;
use crate::inject::InjectFactory;
use crate::proto::seat::service_server::Service as GrpcSeatService;
use crate::proto::seat::{
    CreateSeatsRequest, DeleteSeatsRequest, GetSeatsByRangeRequest, GetSeatsRequest,
    GetSeatsResponse, SeatListResponse, UpdateSeatsRequest,
};
use crate::services::seat::{CreateSeatInput, SeatError, UpdateSeatInput};
use std::sync::Arc;
use tonic::{Request, Response, Status};

pub struct SeatEndpoint {
    factory: Arc<dyn InjectFactory>,
}

impl SeatEndpoint {
    pub fn new(factory: Arc<dyn InjectFactory>) -> Self {
        Self { factory }
    }
}

fn to_status(e: SeatError) -> Status {
    match e {
        SeatError::BusNotFound(id) => {
            Status::invalid_argument(format!("Bus with id {} does not exist", id))
        }
        SeatError::SeatNotFound(id) => {
            Status::not_found(format!("Seat with id {} does not exist", id))
        }
        SeatError::Db(db_err) => Status::internal(db_err.to_string()),
    }
}

#[tonic::async_trait]
impl GrpcSeatService for SeatEndpoint {
    async fn get_seats(
        &self,
        request: Request<GetSeatsRequest>,
    ) -> Result<Response<GetSeatsResponse>, Status> {
        let req = request.into_inner();
        let offset = req.offset.max(0) as u64;
        let size = if req.size <= 0 { 10 } else { req.size } as u64;

        let svc = self.factory.seat_service();
        let (rows, total) = svc
            .get_seats(offset, size)
            .await
            .map_err(|e| Status::internal(e.to_string()))?;

        Ok(Response::new(seat_ctrl::to_get_seats_response(
            &rows, total, offset, size,
        )))
    }

    async fn get_seats_by_range(
        &self,
        request: Request<GetSeatsByRangeRequest>,
    ) -> Result<Response<GetSeatsResponse>, Status> {
        let req = request.into_inner();

        if req.seat_ids.is_empty() && req.names.is_empty() {
            return Ok(Response::new(seat_ctrl::to_get_seats_response(&[], 0, 0, 0)));
        }

        let svc = self.factory.seat_service();
        let rows = svc
            .get_seats_by_range(req.seat_ids, req.names)
            .await
            .map_err(|e| Status::internal(e.to_string()))?;

        let total = rows.len() as u64;
        Ok(Response::new(seat_ctrl::to_get_seats_response(
            &rows, total, 0, total,
        )))
    }

    async fn create_seats(
        &self,
        request: Request<CreateSeatsRequest>,
    ) -> Result<Response<SeatListResponse>, Status> {
        let req = request.into_inner();
        if req.seats.is_empty() {
            return Ok(Response::new(seat_ctrl::to_seat_list_response(
                &[],
                "No seats to create".into(),
            )));
        }

        let inputs: Vec<CreateSeatInput> = req
            .seats
            .into_iter()
            .map(|s| CreateSeatInput {
                bus_id: s.bus_id,
                price: s.price,
                name: s.name,
            })
            .collect();

        let svc = self.factory.seat_service();
        let created = svc.create_seats(&inputs).await.map_err(to_status)?;

        Ok(Response::new(seat_ctrl::to_seat_list_response(
            &created,
            format!("Created {} seat(s)", created.len()),
        )))
    }

    async fn update_seats(
        &self,
        request: Request<UpdateSeatsRequest>,
    ) -> Result<Response<SeatListResponse>, Status> {
        let req = request.into_inner();
        if req.seats.is_empty() {
            return Ok(Response::new(seat_ctrl::to_seat_list_response(
                &[],
                "No seats to update".into(),
            )));
        }

        let inputs: Vec<UpdateSeatInput> = req
            .seats
            .into_iter()
            .map(|s| UpdateSeatInput {
                seat_id: s.seat_id,
                bus_id: s.bus_id,
                price: s.price,
                name: s.name,
            })
            .collect();

        let svc = self.factory.seat_service();
        let updated = svc.update_seats(&inputs).await.map_err(to_status)?;

        Ok(Response::new(seat_ctrl::to_seat_list_response(
            &updated,
            format!("Updated {} seat(s)", updated.len()),
        )))
    }

    async fn delete_seats(
        &self,
        request: Request<DeleteSeatsRequest>,
    ) -> Result<Response<SeatListResponse>, Status> {
        let req = request.into_inner();
        if req.seat_ids.is_empty() {
            return Ok(Response::new(seat_ctrl::to_seat_list_response(
                &[],
                "No seats to delete".into(),
            )));
        }

        let svc = self.factory.seat_service();
        let rows_affected = svc
            .delete_seats(req.seat_ids)
            .await
            .map_err(|e| Status::internal(e.to_string()))?;

        Ok(Response::new(seat_ctrl::to_seat_list_response(
            &[],
            format!("Deleted {} seat(s)", rows_affected),
        )))
    }
}
