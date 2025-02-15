#![allow(elided_lifetimes_in_paths)]
#![allow(clippy::wildcard_imports)]
pub use sea_orm_migration::prelude::*;

mod m20220101_000001_users;

mod m20250202_124347_products;
mod m20250215_190002_postmetas;
pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(m20220101_000001_users::Migration),
            Box::new(m20250202_124347_products::Migration),
            Box::new(m20250215_190002_postmetas::Migration),
            // inject-above (do not remove this comment)
        ]
    }
}