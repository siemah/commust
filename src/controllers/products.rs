#![allow(clippy::missing_errors_doc)]
#![allow(clippy::unnecessary_struct_initialization)]
#![allow(clippy::unused_async)]
use axum::debug_handler;
use axum::{extract::Form, response::Redirect};
use loco_rs::prelude::*;
use sea_orm::{sea_query::Order, QueryOrder};
use serde::{Deserialize, Serialize};
extern crate slug;
use slug::slugify;
use tracing::info;

use crate::{
    models::_entities::products::{ActiveModel, Column, Entity, Model},
    views,
};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Params {
    pub title: String,
    pub excerpt: Option<String>,
    pub status: Option<String>,
    pub product_type: Option<String>,
    pub slug: Option<String>,
}

impl Params {
    fn update(&self, item: &mut ActiveModel) {
        item.title = Set(self.title.clone());
        item.excerpt = Set(self.excerpt.clone());
        item.status = Set(self.status.clone());
        item.product_type = Set(self.product_type.clone());
        // todo: this is just a placeholder so needed to implement
        item.author_id = Set(1);
    }

    fn update_with_slug(&self, item: &mut ActiveModel) {
        item.slug = Set(self.slug.clone());
        item.title = Set(self.title.clone());
        item.excerpt = Set(self.excerpt.clone());
        item.status = Set(self.status.clone());
        item.product_type = Set(self.product_type.clone());
        // todo: this is just a placeholder so needed to implement
        item.author_id = Set(1);
    }
}

async fn load_item(ctx: &AppContext, id: i32) -> Result<Model> {
    let item = Entity::find_by_id(id).one(&ctx.db).await?;
    item.ok_or_else(|| Error::NotFound)
}

#[debug_handler]
pub async fn list(
    ViewEngine(v): ViewEngine<TeraView>,
    State(ctx): State<AppContext>,
) -> Result<Response> {
    let item = Entity::find()
        .order_by(Column::Id, Order::Desc)
        .all(&ctx.db)
        .await?;
    views::products::list(&v, &item)
}

#[debug_handler]
pub async fn new(
    ViewEngine(v): ViewEngine<TeraView>,
    State(_ctx): State<AppContext>,
) -> Result<Response> {
    views::products::create(&v)
}

async fn update_product(ctx: &AppContext, params: Params, id: i32) -> Result<Model> {
    let item = load_item(&ctx, id).await?;
    let mut item = item.into_active_model();
    params.update(&mut item);
    let res = item.update(&ctx.db).await?;

    Ok(res)
}

#[debug_handler]
pub async fn update(
    Path(id): Path<i32>,
    State(ctx): State<AppContext>,
    Form(params): Form<Params>,
) -> Result<Redirect> {
    update_product(&ctx, params, id).await?;
    Ok(Redirect::to("../products"))
}

#[debug_handler]
pub async fn edit(
    Path(id): Path<i32>,
    ViewEngine(v): ViewEngine<TeraView>,
    State(ctx): State<AppContext>,
) -> Result<Response> {
    let item = load_item(&ctx, id).await?;
    views::products::edit(&v, &item)
}

#[debug_handler]
pub async fn show(
    Path(id): Path<i32>,
    ViewEngine(v): ViewEngine<TeraView>,
    State(ctx): State<AppContext>,
) -> Result<Response> {
    let item = load_item(&ctx, id).await?;
    views::products::show(&v, &item)
}

#[debug_handler]
pub async fn show_by_slug(
    Path(slug): Path<String>,
    ViewEngine(v): ViewEngine<TeraView>,
    State(ctx): State<AppContext>,
) -> Result<Response> {
    let item = Entity::find()
        .filter(Column::Slug.eq(slug))
        .one(&ctx.db)
        .await?;
    let product = item.ok_or_else(|| Error::NotFound)?;

    views::products::show(&v, &product)
}

#[debug_handler]
pub async fn add(
    State(ctx): State<AppContext>,
    Form(mut params): Form<Params>,
) -> Result<Redirect> {
    let mut item = ActiveModel {
        ..Default::default()
    };

    params.update(&mut item);

    let res = item.insert(&ctx.db).await?;
    let slug = slugify(format!("{} {}", &params.title, res.id));

    params.slug = Some(slug);

    update_product(&ctx, params, res.id).await?;

    Ok(Redirect::to("products"))
}

#[debug_handler]
pub async fn remove(Path(id): Path<i32>, State(ctx): State<AppContext>) -> Result<Response> {
    load_item(&ctx, id).await?.delete(&ctx.db).await?;
    format::empty()
}

pub fn routes() -> Routes {
    Routes::new()
        .prefix("products/")
        .add("/", get(list))
        .add("/", post(add))
        .add("new", get(new))
        .add(":id", get(show))
        .add("p/:slug", get(show_by_slug))
        .add(":id/edit", get(edit))
        // .add(":id", delete(remove))
        .add(":id", post(update))
}
