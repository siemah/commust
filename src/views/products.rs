use loco_rs::prelude::*;

use crate::models::_entities::products;

/// Render a list view of products.
///
/// # Errors
///
/// When there is an issue with rendering the view.
pub fn list(v: &impl ViewRenderer, items: &Vec<products::Model>) -> Result<Response> {
    format::render().view(v, "products/list.html", data!({"items": items}))
}

/// Render a single products view.
///
/// # Errors
///
/// When there is an issue with rendering the view.
pub fn show(v: &impl ViewRenderer, item: &products::Model) -> Result<Response> {
    format::render().view(v, "products/show.html", data!({"item": item}))
}

/// Render a products create form.
///
/// # Errors
///
/// When there is an issue with rendering the view.
pub fn create(v: &impl ViewRenderer) -> Result<Response> {
    format::render().view(v, "products/create.html", data!({}))
}

/// Render a products edit form.
///
/// # Errors
///
/// When there is an issue with rendering the view.
pub fn edit(v: &impl ViewRenderer, item: &products::Model) -> Result<Response> {
    format::render().view(v, "products/edit.html", data!({"item": item}))
}
