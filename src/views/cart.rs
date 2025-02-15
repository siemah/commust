use crate::controllers::cart::PartialCartProduct;
use loco_rs::prelude::*;

/// Render a cart view.
///
/// # Errors
///
/// When there is an issue with rendering the view.
pub fn show(v: &impl ViewRenderer, items: &Vec<PartialCartProduct>) -> Result<Response> {
    format::render().view(v, "cart/show.html", data!({"items": items}))
}
