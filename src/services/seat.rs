use crate::entities::{bus, seat};
use async_trait::async_trait;
use sea_orm::prelude::Expr;
use sea_orm::*;
use std::fmt;

#[derive(Debug)]
pub enum SeatError {
    BusNotFound(i32),
    SeatNotFound(i32),
    Db(DbErr),
}

impl fmt::Display for SeatError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SeatError::BusNotFound(id) => write!(f, "Bus with id {} does not exist", id),
            SeatError::SeatNotFound(id) => write!(f, "Seat with id {} does not exist", id),
            SeatError::Db(e) => write!(f, "{}", e),
        }
    }
}

impl From<DbErr> for SeatError {
    fn from(e: DbErr) -> Self {
        SeatError::Db(e)
    }
}

pub struct CreateSeatInput {
    pub bus_id: i32,
    pub price: i32,
    pub name: String,
}

pub struct UpdateSeatInput {
    pub seat_id: i32,
    pub bus_id: i32,
    pub price: i32,
    pub name: String,
}

#[async_trait]
pub trait SeatServiceTrait: Send + Sync {
    async fn get_seats(
        &self,
        offset: u64,
        size: u64,
    ) -> Result<(Vec<(seat::Model, Option<bus::Model>)>, u64), DbErr>;

    async fn get_seats_by_range(
        &self,
        seat_ids: Vec<i32>,
        names: Vec<String>,
    ) -> Result<Vec<(seat::Model, Option<bus::Model>)>, DbErr>;

    async fn create_seats(
        &self,
        inputs: &[CreateSeatInput],
    ) -> Result<Vec<seat::Model>, SeatError>;

    async fn update_seats(
        &self,
        inputs: &[UpdateSeatInput],
    ) -> Result<Vec<seat::Model>, SeatError>;

    async fn delete_seats(&self, seat_ids: Vec<i32>) -> Result<u64, DbErr>;
}

pub struct SeatServiceImpl {
    db: DatabaseConnection,
}

impl SeatServiceImpl {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }
}

#[async_trait]
impl SeatServiceTrait for SeatServiceImpl {
    async fn get_seats(
        &self,
        offset: u64,
        size: u64,
    ) -> Result<(Vec<(seat::Model, Option<bus::Model>)>, u64), DbErr> {
        let total = seat::Entity::find()
            .filter(seat::Column::Deleted.eq(false))
            .count(&self.db)
            .await?;

        let rows = seat::Entity::find()
            .filter(seat::Column::Deleted.eq(false))
            .find_also_related(bus::Entity)
            .offset(Some(offset))
            .limit(Some(size))
            .all(&self.db)
            .await?;

        Ok((rows, total))
    }

    async fn get_seats_by_range(
        &self,
        seat_ids: Vec<i32>,
        names: Vec<String>,
    ) -> Result<Vec<(seat::Model, Option<bus::Model>)>, DbErr> {
        let mut condition = Condition::all().add(seat::Column::Deleted.eq(false));

        if !seat_ids.is_empty() {
            condition = condition.add(seat::Column::SeatId.is_in(seat_ids));
        }
        if !names.is_empty() {
            condition = condition.add(seat::Column::Name.is_in(names));
        }

        seat::Entity::find()
            .filter(condition)
            .find_also_related(bus::Entity)
            .all(&self.db)
            .await
    }

    async fn create_seats(
        &self,
        inputs: &[CreateSeatInput],
    ) -> Result<Vec<seat::Model>, SeatError> {
        // Validate bus_ids exist
        let bus_ids: Vec<i32> = inputs.iter().map(|s| s.bus_id).collect();
        let existing_buses = bus::Entity::find()
            .filter(bus::Column::BusId.is_in(bus_ids.clone()))
            .all(&self.db)
            .await?;
        let existing_bus_ids: Vec<i32> = existing_buses.iter().map(|b| b.bus_id).collect();
        for id in &bus_ids {
            if !existing_bus_ids.contains(id) {
                return Err(SeatError::BusNotFound(*id));
            }
        }

        let txn = self.db.begin().await?;

        // Get max seat_id
        let max_id: Option<i32> = seat::Entity::find()
            .select_only()
            .column_as(seat::Column::SeatId.max(), "max_id")
            .into_tuple()
            .one(&txn)
            .await?;
        let mut next_id = max_id.unwrap_or(0) + 1;

        let mut created = Vec::new();
        for input in inputs {
            let model = seat::ActiveModel {
                seat_id: Set(next_id),
                bus_id: Set(input.bus_id),
                price: Set(input.price),
                deleted: Set(false),
                name: Set(input.name.clone()),
            };
            let inserted = seat::Entity::insert(model)
                .exec_with_returning(&txn)
                .await?;
            created.push(inserted);
            next_id += 1;
        }

        txn.commit().await?;

        Ok(created)
    }

    async fn update_seats(
        &self,
        inputs: &[UpdateSeatInput],
    ) -> Result<Vec<seat::Model>, SeatError> {
        // Validate seat_ids exist
        let seat_ids: Vec<i32> = inputs.iter().map(|s| s.seat_id).collect();
        let existing_seats = seat::Entity::find()
            .filter(seat::Column::SeatId.is_in(seat_ids.clone()))
            .filter(seat::Column::Deleted.eq(false))
            .all(&self.db)
            .await?;
        let existing_seat_ids: Vec<i32> = existing_seats.iter().map(|s| s.seat_id).collect();
        for id in &seat_ids {
            if !existing_seat_ids.contains(id) {
                return Err(SeatError::SeatNotFound(*id));
            }
        }

        // Validate bus_ids exist
        let bus_ids: Vec<i32> = inputs.iter().map(|s| s.bus_id).collect();
        let existing_buses = bus::Entity::find()
            .filter(bus::Column::BusId.is_in(bus_ids.clone()))
            .all(&self.db)
            .await?;
        let existing_bus_ids: Vec<i32> = existing_buses.iter().map(|b| b.bus_id).collect();
        for id in &bus_ids {
            if !existing_bus_ids.contains(id) {
                return Err(SeatError::BusNotFound(*id));
            }
        }

        let txn = self.db.begin().await?;

        let mut updated = Vec::new();
        for input in inputs {
            let model = seat::ActiveModel {
                seat_id: Set(input.seat_id),
                bus_id: Set(input.bus_id),
                price: Set(input.price),
                deleted: NotSet,
                name: Set(input.name.clone()),
            };
            let result = seat::Entity::update(model).exec(&txn).await?;
            updated.push(result);
        }

        txn.commit().await?;

        Ok(updated)
    }

    async fn delete_seats(&self, seat_ids: Vec<i32>) -> Result<u64, DbErr> {
        let result = seat::Entity::update_many()
            .col_expr(seat::Column::Deleted, Expr::value(true))
            .filter(seat::Column::SeatId.is_in(seat_ids))
            .filter(seat::Column::Deleted.eq(false))
            .exec(&self.db)
            .await?;

        Ok(result.rows_affected)
    }
}
