use loco_rs::schema::table_auto_tz;
use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                table_auto_tz(Products::Table)
                    .col(pk_auto(Products::Id))
                    .col(string(Products::Title))
                    .col(string_null(Products::Excerpt))
                    .col(string_null(Products::Status))
                    .col(string_null(Products::ProductType))
                    .col(integer(Products::AuthorId))
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk-products-author_ids")
                            .from(Products::Table, Products::AuthorId)
                            .to(Users::Table, Users::Id)
                            .on_delete(ForeignKeyAction::Cascade)
                            .on_update(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Products::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum Products {
    Table,
    Id,
    Title,
    Excerpt,
    Status,
    ProductType,
    AuthorId,
    
}

#[derive(DeriveIden)]
enum Users {
    Table,
    Id,
}
