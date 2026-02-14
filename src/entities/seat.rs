use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "seat")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub seat_id: i32,
    pub bus_id: i32,
    pub price: i32,
    pub deleted: bool,
    pub name: String,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::bus::Entity",
        from = "Column::BusId",
        to = "super::bus::Column::BusId"
    )]
    Bus,
}

impl Related<super::bus::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Bus.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
