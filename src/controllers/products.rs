#![allow(clippy::missing_errors_doc)]
#![allow(clippy::unnecessary_struct_initialization)]
#![allow(clippy::unused_async)]
use axum::debug_handler;
use axum::{extract::Form, response::Redirect};
use axum_session::{Session, SessionNullPool};
use loco_rs::prelude::*;
use migration::{Expr};
use sea_orm::{FromQueryResult, QuerySelect, UpdateResult};
use sea_orm::{sea_query::Order, QueryOrder};
use serde::{Deserialize, Serialize};
extern crate slug;
use slug::slugify;
use tracing::info;
use axum_extra::extract::Form as FormExtra;
use crate::{
    models::_entities::products::{ActiveModel, Column, Entity, Model},
    views,
};
use crate::models::_entities::postmetas::{self, ActiveModel as PmActiveModel, Entity as PmEntity};
use crate::helpers::serdes::{empty_string_as_none, from_attributes_values};

#[derive(Clone, Debug, Deserialize)]
pub struct Params {
    pub title: String,
    pub excerpt: Option<String>,
    pub status: Option<String>,
    pub product_type: Option<String>,
    pub slug: Option<String>,

    // postmetas fields
    pub _sku: Option<String>,
    #[serde(deserialize_with = "empty_string_as_none")]
    pub _regular_price: Option<f32>,
    #[serde(deserialize_with = "empty_string_as_none")]
    pub _sale_price: Option<String>,
    #[serde(deserialize_with = "empty_string_as_none")]
    pub _stock: Option<f32>,
    
    // attributes
    #[serde(default)]
    pub attributes_names: Vec<String>,
    #[serde(default, deserialize_with = "from_attributes_values")]
    pub attributes_values: Vec<Vec<String>>,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
struct AttributeNames {
    attributes_names: Vec<String>,
}

impl Params {
    fn update(&self, item: &mut ActiveModel) {
        item.title = Set(self.title.clone());
        item.excerpt = Set(self.excerpt.clone());
        item.status = Set(self.status.clone());
        item.product_type = Set(self.product_type.clone());
        // todo: this is just a placeholder so it must be implemented properly
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

async fn generate_product_slug(ctx: &AppContext, id: i32, title: &String) -> Result<UpdateResult> {
    let slug = slugify(format!("{} {}", title, id));
    let res = Entity::update_many()
        .col_expr(Column::Slug, Expr::value(slug))
        .filter(Column::Id.eq(id))
        .exec(&ctx.db)
        .await?;

    Ok(res)
}

async fn save_product_meta(ctx: &AppContext, id: i32, params: Params) -> Result<()> {
    let mut meta_data:Vec<PmActiveModel> = vec![];

    if params._regular_price.is_some() {
        let old_price = PmEntity::find()
            .filter(postmetas::Column::ProductId.eq(id))
            .filter(postmetas::Column::MetaKey.eq("_regular_price"))
            .one(&ctx.db)
            .await?;

        if old_price.is_none() {
            let meta_price = PmActiveModel {
                product_id: Set(id),
                meta_key: Set(Some("_regular_price".to_string())),
                meta_value: Set(Some(params._regular_price.unwrap().to_string())),
                ..Default::default()
            };
            meta_data.push(meta_price);
        } else {
            let mut old_price: PmActiveModel = old_price.unwrap().into();
            old_price.meta_value = Set(Some(params._regular_price.unwrap().to_string()));
            old_price.update(&ctx.db).await?;
        }
    }

    if params._sale_price.is_some() {
        let old_sale_price = PmEntity::find()
            .filter(postmetas::Column::ProductId.eq(id))
            .filter(postmetas::Column::MetaKey.eq("_sale_price"))
            .one(&ctx.db)
            .await?;

        if old_sale_price.is_none() {
            let meta_price = PmActiveModel {
                product_id: Set(id),
                meta_key: Set(Some("_sale_price".to_string())),
                meta_value: Set(Some(params._sale_price.unwrap().to_string())),
                ..Default::default()
            };
            meta_data.push(meta_price);
        } else {
            let mut old_sale_price: PmActiveModel = old_sale_price.unwrap().into();
            old_sale_price.meta_value = Set(Some(params._sale_price.unwrap().to_string()));
            old_sale_price.update(&ctx.db).await?;
        }
    }

    if params._sku.is_some() {
        let old_sku = PmEntity::find()
            .filter(postmetas::Column::ProductId.eq(id))
            .filter(postmetas::Column::MetaKey.eq("_sku"))
            .one(&ctx.db)
            .await?;

        if old_sku.is_none() {
            let meta_sku = PmActiveModel {
                product_id: Set(id),
                meta_key: Set(Some("_sku".to_string())),
                meta_value: Set(params._sku),
                ..Default::default()
            };
            meta_data.push(meta_sku);
        } else {
            let mut old_sku: PmActiveModel = old_sku.unwrap().into();
            old_sku.meta_value = Set(params._sku);
            old_sku.update(&ctx.db).await?;
        }
    }
    
    let old_stock_status = PmEntity::find()
        .filter(postmetas::Column::ProductId.eq(id))
        .filter(postmetas::Column::MetaKey.eq("_stock_status"))
        .one(&ctx.db)
        .await?;
    let old_manage_stock = PmEntity::find()
        .filter(postmetas::Column::ProductId.eq(id))
        .filter(postmetas::Column::MetaKey.eq("_manage_stock"))
        .one(&ctx.db)
        .await?;
    let old_stock = PmEntity::find()
        .filter(postmetas::Column::ProductId.eq(id))
        .filter(postmetas::Column::MetaKey.eq("_stock"))
        .one(&ctx.db)
        .await?;

    if params._stock.is_some() {
        let stock = params._stock.unwrap();
        let stock_status = if stock > 0.0 {
            "instock".to_string()
        } else {
            "outofstock".to_string()
        };
        
        if old_stock.is_none() {
            let meta_stock = PmActiveModel {
                product_id: Set(id),
                meta_key: Set(Some("_stock".to_string())),
                meta_value: Set(Some(stock.to_string())),
                ..Default::default()
            };
            meta_data.push(meta_stock);
        } else {
            let mut old_stock: PmActiveModel = old_stock.unwrap().into();
            old_stock.meta_value = Set(Some(stock.to_string()));
            old_stock.update(&ctx.db).await?;
        }
        
        let old_stock_status = PmEntity::find()
            .filter(postmetas::Column::ProductId.eq(id))
            .filter(postmetas::Column::MetaKey.eq("_stock_status"))
            .one(&ctx.db)
            .await?;

        if old_stock_status.is_none() {
            let meta_status_stock = PmActiveModel {
                product_id: Set(id),
                meta_key: Set(Some("_stock_status".to_string())),
                meta_value: Set(Some(stock_status)),
                ..Default::default()
            };
            meta_data.push(meta_status_stock);
        } else {
            let mut old_stock_status: PmActiveModel = old_stock_status.unwrap().into();
            old_stock_status.meta_value = Set(Some(stock_status));
            old_stock_status.update(&ctx.db).await?;
        }

        if old_manage_stock.is_none() {
            let meta_manage_stock = PmActiveModel {
                product_id: Set(id),
                meta_key: Set(Some("_manage_stock".to_string())),
                meta_value: Set(Some(true.to_string())),
                ..Default::default()
            };
            meta_data.push(meta_manage_stock);
        } else {
            let mut old_manage_stock: PmActiveModel = old_manage_stock.unwrap().into();
            old_manage_stock.meta_value = Set(Some(true.to_string()));
            old_manage_stock.update(&ctx.db).await?;
        }
    } else {
        // delete _stock to avoid parsing error of an empty string
        if old_stock.is_some() {
            let old_stock: PmActiveModel = old_stock.unwrap().into();
            old_stock.delete(&ctx.db).await?;
        }

        if old_manage_stock.is_none() {
            let meta_manage_stock = PmActiveModel {
                product_id: Set(id),
                meta_key: Set(Some("_manage_stock".to_string())),
                meta_value: Set(Some(false.to_string())),
                ..Default::default()
            };
            meta_data.push(meta_manage_stock);
        } else {
            let mut old_manage_stock: PmActiveModel = old_manage_stock.unwrap().into();
            old_manage_stock.meta_value = Set(Some(false.to_string()));
            old_manage_stock.update(&ctx.db).await?;
        }
        
        if old_stock_status.is_none() {
            let meta_status_stock = PmActiveModel {
                product_id: Set(id),
                meta_key: Set(Some("_stock_status".to_string())),
                meta_value: Set(Some("onbackorder".to_string())),
                ..Default::default()
            };
            meta_data.push(meta_status_stock);
        } else {
            let mut old_stock_status: PmActiveModel = old_stock_status.unwrap().into();
            old_stock_status.meta_value = Set(Some("onbackorder".to_string()));
            old_stock_status.update(&ctx.db).await?;
        }
    }


    // todo: save attributes
    let attribute_name = "_product_attributes";
    let old_attribute = PmEntity::find()
        .filter(postmetas::Column::ProductId.eq(id))
        .filter(postmetas::Column::MetaKey.eq(attribute_name))
        .one(&ctx.db)
        .await?;
    
    if params.attributes_names.len() == params.attributes_values.len() && params.attributes_names.len() > 0 {
        let mut meta_value = vec![];
        for (i, name) in params.attributes_names.iter().enumerate() {
            let value = &params.attributes_values[i];
            meta_value.push(
                data!({
                    "name": name,
                    "slug": slugify(name),
                    "position": i,
                    "is_visible": true,
                    "is_variation": false,
                    "is_taxonomy": false,
                    "value": value,
                })
            ); 
        }
           
        if old_attribute.is_none() {
            let meta_attribute = PmActiveModel {
                product_id: Set(id),
                meta_key: Set(Some(attribute_name.to_string())),
                meta_value: Set(Some(data!(meta_value).to_string())),
                ..Default::default()
            };
            meta_data.push(meta_attribute);
        } else {
            let mut old_attribute: PmActiveModel = old_attribute.unwrap().into();
            old_attribute.meta_value = Set(Some(data!(meta_value).to_string()));
            old_attribute.update(&ctx.db).await?;
        }
    } else if old_attribute.is_some() {
        let old_attribute: PmActiveModel = old_attribute.unwrap().into();
        old_attribute.delete(&ctx.db).await?;
    }

    if meta_data.len() > 0 {
        PmEntity::insert_many(meta_data).exec(&ctx.db).await?;
    }

    Ok(())
}

#[debug_handler]
pub async fn update(
    Path(id): Path<i32>,
    State(ctx): State<AppContext>,
    Form(params): Form<Params>,
) -> Result<Redirect> {
    let mut item = load_item(&ctx, id).await?.into_active_model();
    params.update(&mut item);
    item.update(&ctx.db).await?;

    save_product_meta(&ctx, id, params).await?;
    info!("Product updated {:?}", id);

    let redirect_url = format!("/products/{}/edit", id); 

    Ok(Redirect::to(redirect_url.as_str()))
}

#[debug_handler]
pub async fn edit(
    Path(id): Path<i32>,
    ViewEngine(v): ViewEngine<TeraView>,
    State(ctx): State<AppContext>,
) -> Result<Response> {
    let item = load_item(&ctx, id).await?;
    let meta_data = item.find_related(PmEntity)
        .select_only()
        .column(postmetas::Column::Id)
        .column(postmetas::Column::MetaKey)
        .column(postmetas::Column::MetaValue)
        .into_model::<PartialMetaModel>()
        .all(&ctx.db) 
        .await?;

    let product = ProductView::build(item, meta_data);

    views::products::edit(&v, &product)
}

#[derive(FromQueryResult)]
struct PartialMetaModel {
    // pub id: i32,
    pub meta_key: Option<String>,
    pub meta_value: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct ProductView{
    pub id: i32,
    pub name: String,
    pub slug: String,
    pub product_type: String,
    pub excerpt: String,
    pub status: String,
    
    // meta data
    pub sku: Option<String>,
    pub regular_price: Option<f32>,
    pub sale_price: Option<f32>,
    pub price: Option<f32>,
    pub stock: Option<f32>,
    pub stock_status: String,
    pub attributes: Vec<ProductAttribute>,
}

#[derive(Deserialize, Serialize, Debug)]
struct ProductAttribute {
    name: String,
    slug: String,
    position: u8,
    #[serde(rename = "is_visible")]
    visible: bool,
    #[serde(rename = "is_variation")]
    variation: bool,
    #[serde(rename = "value")]
    options: Vec<String>,
}

impl Default for ProductView {
    fn default() -> Self {
        Self {
            id: 0,
            name: "".to_string(),
            slug: "".to_string(),
            product_type: "simple".to_string(),
            excerpt: "".to_string(),
            status: "draft".to_string(),
            sku: None,
            regular_price: None,
            sale_price: None,
            price: None,
            stock: None,
            stock_status: "".to_string(),
            attributes: vec![],
        }
    } 
}

impl ProductView {
    fn build(model: Model, meta_data: Vec<PartialMetaModel>) -> Self {
        let mut product = ProductView::default();
        product.id = model.id;
        product.name = model.title;
        product.slug = model.slug.unwrap_or("".to_string());
        product.excerpt = model.excerpt.unwrap_or("".to_string());
        product.status = model.status.unwrap_or("".to_string());
        product.product_type = model.product_type.unwrap_or("".to_string());

        for meta in meta_data {
            match meta.meta_key.as_deref() {
                Some("_sku") => {
                    product.sku = meta.meta_value;
                }
                Some("_regular_price") => {
                    product.regular_price = Some(meta.meta_value.unwrap().parse().unwrap());
                }
                Some("_sale_price") => {
                    product.sale_price = Some(meta.meta_value.unwrap().parse().unwrap());
                }
                Some("_stock") => {
                    product.stock = Some(meta.meta_value.unwrap().parse().unwrap());
                }
                Some("_stock_status") => {
                    product.stock_status = meta.meta_value.unwrap();
                }
                Some("_product_attributes") => { 
                    let attributes:Vec<ProductAttribute> = serde_json::from_str(
                            meta.meta_value
                                .unwrap()
                                .as_str()
                        )
                        .unwrap_or(vec![]);
                    product.attributes = attributes;
                },
                _ => {}
            }
        }

        product.price = if product.sale_price.is_some() {
            product.sale_price
        } else if product.regular_price.is_some() {
            product.regular_price
        } else {
            None
        };

        product
    }
}

#[debug_handler]
pub async fn show(
    Path(id): Path<i32>,
    session: Session<SessionNullPool>,
    ViewEngine(v): ViewEngine<TeraView>,
    State(ctx): State<AppContext>,
) -> Result<Response> {
    let item = load_item(&ctx, id).await?;
    // todo: map meta_data to a object like when mapping remove _ from meta_key converting from
    // vectore/array to object using data! or json! macro from serde
     
    // todo: merge item and meta_data object into one object
    let errors = session.get::<serde_json::Value>("errors").unwrap_or(data!({}));
    session.set("errors", data!({}));
    
    views::products::show(&v, &item, &errors)
}

#[debug_handler]
pub async fn show_by_slug(
    Path(slug): Path<String>,
    session: Session<SessionNullPool>,
    ViewEngine(v): ViewEngine<TeraView>,
    State(ctx): State<AppContext>,
) -> Result<Response> {
    let item = Entity::find()
        .filter(Column::Slug.eq(slug))
        .one(&ctx.db)
        .await?;
    let product = item.ok_or_else(|| Error::NotFound)?;
    let errors = session.get::<serde_json::Value>("errors").unwrap_or(data!({}));
    session.set("errors", data!({}));

    views::products::show(&v, &product, &errors)
}

#[debug_handler]
pub async fn add(
    State(ctx): State<AppContext>,
    FormExtra(params): FormExtra<Params>,
) -> Result<Redirect> {
    let mut item = ActiveModel {
        ..Default::default()
    };

    params.update(&mut item);

    let res = item.insert(&ctx.db).await?;

    generate_product_slug(&ctx, res.id, &params.title).await?;
    save_product_meta(&ctx, res.id, params).await?;
    
    info!("Product added: {:#?}", res);
    
    let redirect_url = format!("/products/{}/edit", res.id);
    
    Ok(Redirect::to(redirect_url.as_str()))
}

#[debug_handler]
pub async fn remove(Path(id): Path<i32>, State(ctx): State<AppContext>) -> Result<Response> {
    load_item(&ctx, id).await?.delete(&ctx.db).await?;
    info!("Product removed: {}", id);
    
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
        .add(":id", delete(remove))
        .add(":id", post(update))
}
