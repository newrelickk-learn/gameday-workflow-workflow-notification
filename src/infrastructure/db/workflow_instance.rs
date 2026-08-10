use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq, Serialize, Deserialize)]
#[sea_orm(table_name = "workflow_instances")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Uuid,
    #[sea_orm(unique)]
    pub application_id: String,
    pub workflow_definition_id: Uuid,
    pub current_step: i32,
    pub status: String,
    #[sea_orm(column_type = "TimestampWithTimeZone")]
    pub created_at: DateTimeWithTimeZone,
    #[sea_orm(column_type = "TimestampWithTimeZone")]
    pub updated_at: DateTimeWithTimeZone,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::workflow_definition::Entity",
        from = "Column::WorkflowDefinitionId",
        to = "super::workflow_definition::Column::Id"
    )]
    WorkflowDefinition,
}

impl Related<super::workflow_definition::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::WorkflowDefinition.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}

