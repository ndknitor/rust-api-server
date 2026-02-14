use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "bus")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub bus_id: i32,
    pub name: Option<String>,
    pub license_plate: String,
    pub deleted: bool,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(has_many = "super::seat::Entity")]
    Seat,
}

impl Related<super::seat::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Seat.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
