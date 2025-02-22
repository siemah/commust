#![allow(clippy::missing_errors_doc)]
#![allow(clippy::unnecessary_struct_initialization)]
#![allow(clippy::unused_async)]
use axum::{debug_handler, extract::Form, response::Redirect};
use axum_extra::extract::CookieJar;
use axum_session::{Session, SessionNullPool};
use loco_rs::prelude::{cookie::Cookie, Uuid, *};
use sea_orm::{FromQueryResult, Order, QueryOrder, QuerySelect};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use tracing::info;

use crate::{
    models::{_entities::{postmetas, products::{Column, Entity}}},
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

#[derive(FromQueryResult)]
struct PartialMetaModel {
    // pub id: i32,
    pub meta_key: Option<String>,
    pub meta_value: Option<String>,
}

#[debug_handler]
pub async fn add(
    session: Session<SessionNullPool>,
    jar: CookieJar,
    State(ctx): State<AppContext>,
    Form(params): Form<CartParams>,
) -> Result<(CookieJar, Redirect)> {
    let redirect_to = format!("/products/p/{}", &params.slug);
    let item = postmetas::Entity::find()
        .select_only()
        .column(postmetas::Column::MetaKey)
        .column(postmetas::Column::MetaValue)
        .filter(postmetas::Column::MetaKey.eq("_stock_status"))
        .filter(postmetas::Column::ProductId.eq(params.id))
        .into_model::<PartialMetaModel>()
        .one(&ctx.db)
        .await?;
    
    // this check if the product exists 
    if item.is_none() {
        return Ok((jar, Redirect::to("/products")));
    } 
    
    let stock_status = item.unwrap().meta_value.unwrap();
    
    if stock_status == "outofstock" {
        // todo: add a flash message to the session
        let errors = serde_json::json!({
            "global": "The requested quantity is not available",
        });
        session.set("errors", errors);

        return Ok((jar, Redirect::to(redirect_to.as_str())));
    }

    let mut cart_session: Vec<CartSession> = session.get("commust_cart_items").unwrap_or(vec![]);
    let item_position = cart_session.iter().position(|x| x.id == params.id);

    if stock_status == "instock" {
        let stock_qty = postmetas::Entity::find()
            .select_only()
            .column(postmetas::Column::MetaValue)
            .filter(postmetas::Column::MetaKey.eq("_stock"))
            .filter(postmetas::Column::ProductId.eq(params.id))
            .into_model::<PartialMetaModel>()
            .one(&ctx.db)
            .await?;
        let stock_qty = stock_qty.unwrap().meta_value.unwrap().parse::<i32>().unwrap();
        
        // note: add current qty(already in cart) to the requested qty as well
        
        let current_qty = if item_position.is_some() {
            let index = item_position.unwrap();
            cart_session[index].qty
        } else {
            0
        };

        if stock_qty < params.qty + current_qty {
            // todo: set error in session
            let errors = serde_json::json!({
                "global": "The requested quantity is not available",
            });
            session.set("errors", errors);

            return Ok((jar, Redirect::to(redirect_to.as_str())));
        }
    }

    // todo: check if variations are valid

    let session_id = match jar.get("commust_session_id") {
        Some(cookie) => cookie.value().to_string(),
        None => Uuid::new_v4().to_string(),
    };
    
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
    pub key: String,
    pub id: i32,
    pub slug: Option<String>,
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
            let current_cart_item = cart_session.iter().find(|x| x.id == product.id).unwrap();
            PartialCartProduct {
                key: current_cart_item.key.clone(),
                id: product.id,
                slug: product.slug,
                name: product.name,
                quantity: current_cart_item.qty,
            }
        })
        .collect::<Vec<PartialCartProduct>>();

    views::cart::show(&v, &products)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CartRemoveItemParams {
    pub key: String,
}

#[debug_handler]
pub async fn remove(
    session: Session<SessionNullPool>,
    jar: CookieJar,
    State(_ctx): State<AppContext>,
    Form(params): Form<CartRemoveItemParams>,
) -> Result<(CookieJar, Redirect)> {
    let session_id = match jar.get("commust_session_id") {
        Some(cookie) => cookie.value().to_string(),
        None => Uuid::new_v4().to_string(),
    };
    let mut cart_session: Vec<CartSession> = session.get("commust_cart_items").unwrap_or(vec![]);

    cart_session = cart_session
        .into_iter()
        .filter(|x| x.key != params.key)
        .collect();

    let items = cart_session.len();
    let cart_hash = generate_hash(&cart_session, &session_id);
    session.set("commust_cart_items", cart_session);

    info!("Cart item {} removed from cart", params.key);

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
    let redirect_to = format!("/cart");

    Ok((
        // the updated jar must be returned for the changes
        // to be included in the response
        jar.add(hash_cookie).add(items_cookie).add(session_cookie),
        Redirect::to(redirect_to.as_str()),
    ))
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CartUpdateItemParams {
    pub key: String,
    pub qty: i32,
}

#[debug_handler]
pub async fn update(
    session: Session<SessionNullPool>,
    jar: CookieJar,
    State(_ctx): State<AppContext>,
    Form(params): Form<CartUpdateItemParams>,
) -> Result<(CookieJar, Redirect)> {
    let session_id = match jar.get("commust_session_id") {
        Some(cookie) => cookie.value().to_string(),
        None => Uuid::new_v4().to_string(),
    };
    let mut cart_session: Vec<CartSession> = session.get("commust_cart_items").unwrap_or(vec![]);

    let item_position = cart_session.iter().position(|x| x.key == params.key);

    if item_position.is_some() {
        let index = item_position.unwrap();
        cart_session[index].qty = params.qty;
    }

    let items = cart_session.len();
    let cart_hash = generate_hash(&cart_session, &session_id);
    session.set("commust_cart_items", cart_session);

    info!("Cart item {} updated its quantity to {}", params.key, params.qty);

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
    let redirect_to = format!("/cart");

    Ok((
        // the updated jar must be returned for the changes
        // to be included in the response
        jar.add(hash_cookie).add(items_cookie).add(session_cookie),
        Redirect::to(redirect_to.as_str()),
    ))
}

pub fn routes() -> Routes {
    Routes::new()
        .prefix("cart/")
        .add("/", get(show))
        .add("add-item", post(add))
        .add("remove-item", post(remove))
        .add("update-item", post(update))
}
