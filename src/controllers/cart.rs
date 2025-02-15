#![allow(clippy::missing_errors_doc)]
#![allow(clippy::unnecessary_struct_initialization)]
#![allow(clippy::unused_async)]
use axum::{debug_handler, extract::Form, response::Redirect};
use axum_extra::extract::CookieJar;
use axum_session::{Session, SessionNullPool};
use loco_rs::prelude::{cookie::Cookie, Uuid, *};
use sea_orm::{FromQueryResult, Order, QueryOrder, QuerySelect, SelectColumns};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use tracing::info;

use crate::{
    models::_entities::products::{Column, Entity},
    views,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CartParams {
    pub id: i32,
    pub qty: i32,
    pub slug: String,
    // pub variations: Option<String>,
}

fn generate_hash<'a>(params: &Vec<CartSession>, session_id: &'a String) -> String {
    let mut content = format!("{}:", session_id);

    for item in params {
        content = format!("{}{}*{}+", content, item.id, item.qty);
    }
    content.pop();

    info!("generate cart hash");

    let mut hasher = Sha256::new();
    hasher.update(content.as_str());
    format!("{:x}", hasher.finalize())
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CartSession {
    key: String,
    id: i32,
    qty: i32,
    // variations: Option<Vec>,
}

#[debug_handler]
pub async fn add(
    session: Session<SessionNullPool>,
    jar: CookieJar,
    State(_ctx): State<AppContext>,
    Form(params): Form<CartParams>,
) -> Result<(CookieJar, Redirect)> {
    let session_id = match jar.get("commust_session_id") {
        Some(cookie) => cookie.value().to_string(),
        None => Uuid::new_v4().to_string(),
    };
    let mut cart_session: Vec<CartSession> = session.get("commust_cart_items").unwrap_or(vec![]);

    let item_position = cart_session.iter().position(|x| x.id == params.id);

    if item_position.is_some() {
        let index = item_position.unwrap();
        cart_session[index].qty += params.qty;
    } else {
        let new_cart_item = CartSession {
            key: Uuid::new_v4().to_string(),
            id: params.id,
            qty: params.qty,
        };
        cart_session.push(new_cart_item);
    }

    let items = cart_session.len();
    let cart_hash = generate_hash(&cart_session, &session_id);
    println!("{:#?}", cart_session);
    session.set("commust_cart_items", cart_session);

    info!("Product {} added {} times to cart", params.id, params.qty);

    let hash_cookie = Cookie::build(("commust_cart_hash", cart_hash))
        .path("/")
        .http_only(true)
        .secure(false);
    let items_cookie = Cookie::build(("commust_cart_items", items.to_string()))
        .path("/")
        .http_only(true)
        .secure(false);
    let session_cookie = Cookie::build(("commust_session_id", session_id.to_string()))
        .path("/")
        .http_only(true)
        .secure(false);
    let redirect_to = format!("/products/p/{}", &params.slug);

    Ok((
        // the updated jar must be returned for the changes
        // to be included in the response
        jar.add(hash_cookie).add(items_cookie).add(session_cookie),
        Redirect::to(redirect_to.as_str()),
    ))
}

#[derive(FromQueryResult)]
struct PartialProductModel {
    pub id: i32,
    pub name: String,
    pub slug: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct PartialCartProduct {
    pub id: i32,
    pub slug: String,
    pub name: String,
    pub quantity: i32,
}

#[debug_handler]
pub async fn show(
    session: Session<SessionNullPool>,
    ViewEngine(v): ViewEngine<TeraView>,
    State(ctx): State<AppContext>,
) -> Result<Response> {
    let cart_session: Vec<CartSession> = session.get("commust_cart_items").unwrap_or(vec![]);
    let ids = cart_session.iter().map(|x| x.id).collect::<Vec<i32>>();
    let products_list = Entity::find()
        .select_only()
        .column(Column::Id)
        .column(Column::Slug)
        .column_as(Column::Title, "name")
        .order_by(Column::Id, Order::Asc)
        .filter(Column::Id.is_in(ids))
        .into_model::<PartialProductModel>()
        .all(&ctx.db)
        .await?;
    let products = products_list
        .into_iter()
        .map(|product| {
            let quantity = cart_session
                .iter()
                .find(|x| x.id == product.id)
                .unwrap()
                .qty;
            PartialCartProduct {
                id: product.id,
                slug: product.slug.unwrap(),
                name: product.name,
                quantity,
            }
        })
        .collect::<Vec<PartialCartProduct>>();

    println!("{:#?}", products);
    views::cart::show(&v, &products)
}

pub fn routes() -> Routes {
    Routes::new()
        .prefix("cart/")
        .add("/", get(show))
        .add("add-item", post(add))
}
