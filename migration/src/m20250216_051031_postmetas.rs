use loco_rs::schema::table_auto_tz;
use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                table_auto_tz(Postmetas::Table)
                    .col(pk_auto(Postmetas::Id))
                    .col(string_null(Postmetas::MetaKey))
                    .col(text_null(Postmetas::MetaValue))
                    .col(integer(Postmetas::ProductId))
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk-postmetas-products_ids")
                            .from(Postmetas::Table, Postmetas::ProductId)
                            .to(Products::Table, Products::Id)
                            .on_delete(ForeignKeyAction::Cascade)
                            .on_update(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Postmetas::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum Postmetas {
    Table,
    Id,
    MetaKey,
    MetaValue,
    ProductId,    
}

#[derive(DeriveIden)]
enum Products {
    Table,
    Id,
}
