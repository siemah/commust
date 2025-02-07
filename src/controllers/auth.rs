use crate::{
    mailers::auth::AuthMailer,
    models::{
        _entities::users,
        users::{LoginParams, RegisterParams},
    },
    views,
    views::auth::{CurrentResponse, LoginResponse},
};
use axum::debug_handler;
use axum::{extract::Form, response::Redirect};
use axum_session::{Session, SessionNullPool};
use loco_rs::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct VerifyParams {
    pub token: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ForgotParams {
    pub email: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ResetParams {
    pub token: String,
    pub password: String,
}

#[debug_handler]
pub async fn register_view(ViewEngine(v): ViewEngine<TeraView>) -> Result<Response> {
    views::auth::register_view(&v)
}

#[debug_handler]
pub async fn login_view(
    ViewEngine(v): ViewEngine<TeraView>,
    session: Session<SessionNullPool>,
) -> Result<Response> {
    // todo: check if user is already logged in
    // todo: redirect to dashboard if user is already logged in
    // todo: pass error message to the view
    let errors = session
        .get::<serde_json::Value>("errors")
        .unwrap_or(data!({}));
    session.set("errors", data!({}));

    views::auth::login_view(&v, &errors)
}

/// Register function creates a new user with the given parameters and sends a
/// welcome email to the user
#[debug_handler]
async fn register(ctx: State<AppContext>, Json(params): Json<RegisterParams>) -> Result<Response> {
    tracing::trace!("here we go ");
    let res = add_user(ctx, Form(params)).await?;

    match res.status {
        ResponseStatus::Created => {
            // return a success message and data of user without password field
            return format::json(res.data);
        }
        _ => {
            let message = res.message.unwrap_or("Something went wrong".to_string());
            return format::json(message);
        }
    };
}

/// Verify register user. if the user not verified his email, he can't login to
/// the system.
#[debug_handler]
async fn verify(
    State(ctx): State<AppContext>,
    Json(params): Json<VerifyParams>,
) -> Result<Response> {
    let user = users::Model::find_by_verification_token(&ctx.db, &params.token).await?;

    if user.email_verified_at.is_some() {
        tracing::info!(pid = user.pid.to_string(), "user already verified");
    } else {
        let active_model = user.into_active_model();
        let user = active_model.verified(&ctx.db).await?;
        tracing::info!(pid = user.pid.to_string(), "user verified");
    }

    format::json(())
}

/// In case the user forgot his password  this endpoints generate a forgot token
/// and send email to the user. In case the email not found in our DB, we are
/// returning a valid request for for security reasons (not exposing users DB
/// list).
#[debug_handler]
async fn forgot(
    State(ctx): State<AppContext>,
    Json(params): Json<ForgotParams>,
) -> Result<Response> {
    let Ok(user) = users::Model::find_by_email(&ctx.db, &params.email).await else {
        // we don't want to expose our users email. if the email is invalid we still
        // returning success to the caller
        return format::json(());
    };

    let user = user
        .into_active_model()
        .set_forgot_password_sent(&ctx.db)
        .await?;

    AuthMailer::forgot_password(&ctx, &user).await?;

    format::json(())
}

/// reset user password by the given parameters
#[debug_handler]
async fn reset(State(ctx): State<AppContext>, Json(params): Json<ResetParams>) -> Result<Response> {
    let Ok(user) = users::Model::find_by_reset_token(&ctx.db, &params.token).await else {
        // we don't want to expose our users email. if the email is invalid we still
        // returning success to the caller
        tracing::info!("reset token not found");

        return format::json(());
    };
    user.into_active_model()
        .reset_password(&ctx.db, &params.password)
        .await?;

    format::json(())
}

/// Creates a user login and returns a token
#[debug_handler]
async fn login(
    // auth: auth::JWT,
    State(ctx): State<AppContext>,
    Json(params): Json<LoginParams>,
) -> Result<Response> {
    let user = users::Model::find_by_email(&ctx.db, &params.email).await?;

    let valid = user.verify_password(&params.password);

    if !valid {
        return unauthorized("unauthorized!");
    }

    let jwt_secret = ctx.config.get_jwt_config()?;

    let token = user
        .generate_jwt(&jwt_secret.secret, &jwt_secret.expiration)
        .or_else(|_| unauthorized("unauthorized!"))?;
    // save token in a cookie header of the response

    format::json(LoginResponse::new(&user, &token))
}

#[debug_handler]
async fn current(auth: auth::JWT, State(ctx): State<AppContext>) -> Result<Response> {
    let user = users::Model::find_by_pid(&ctx.db, &auth.claims.pid).await?;
    format::json(CurrentResponse::new(&user))
}

#[warn(dead_code)]
async fn old_register(
    State(ctx): State<AppContext>,
    Json(params): Json<RegisterParams>,
) -> Result<Response> {
    let res = users::Model::create_with_password(&ctx.db, &params).await;

    let user = match res {
        Ok(user) => user,
        Err(err) => {
            tracing::info!(
                message = err.to_string(),
                user_email = &params.email,
                "could not register user",
            );
            return format::json(());
        }
    };

    // let user = user
    //     .into_active_model()
    //     .set_email_verification_sent(&ctx.db)
    //     .await?;

    format::json(())
}

#[derive(Debug, Serialize)]
enum ResponseStatus {
    Created,
    BadRequest,
}
#[derive(Debug, Serialize)]
struct HttpResponse {
    status: ResponseStatus,
    data: serde_json::Value,
    message: Option<String>,
}

async fn add_user(
    State(ctx): State<AppContext>,
    Form(params): Form<RegisterParams>,
) -> Result<HttpResponse> {
    let res = users::Model::create_with_password(&ctx.db, &params).await;
    let response: HttpResponse;
    let user = match res {
        Ok(user) => user,
        Err(err) => {
            tracing::info!(
                message = err.to_string(),
                user_email = &params.email,
                "could not register user",
            );
            response = HttpResponse {
                status: ResponseStatus::BadRequest,
                data: serde_json::json!({ "error": err.to_string() }),
                message: Some(err.to_string()),
            };
            return Ok(response);
        }
    };

    let user = user
        .into_active_model()
        .set_email_verification_sent(&ctx.db)
        .await?;

    // todo: config smtp to send email
    // AuthMailer::send_welcome(&ctx, &user).await?;

    let data = serde_json::json!({
       "name": user.name,
       "email": user.email,
    });
    response = HttpResponse {
        status: ResponseStatus::Created,
        data,
        message: None,
    };

    Ok(response)
}

async fn register_via_form(
    session: Session<SessionNullPool>,
    ctx: State<AppContext>,
    params: Form<RegisterParams>,
) -> Result<Redirect> {
    let res = add_user(ctx, params).await?;
    match res.status {
        ResponseStatus::Created => {
            // return a success message and data of user without password field
            return Ok(Redirect::to("/auth/login"));
        }
        _ => {
            let message = res.message.unwrap_or("Something went wrong".to_string());
            let errors = serde_json::json!({
                "email": message,
            });
            session.set("errors", errors);

            return Ok(Redirect::to("/auth/register"));
        }
    };
}

async fn login_via_form(
    session: Session<SessionNullPool>,
    ctx: State<AppContext>,
    params: Form<LoginParams>,
) -> Result<Redirect> {
    let user = users::Model::find_by_email(&ctx.db, &params.email).await?;

    let valid = user.verify_password(&params.password);

    if !valid {
        let errors = serde_json::json!({
            "email": "invalid email or password",
            "password": "invalid email or password",
        });
        session.set("errors", errors);

        return Ok(Redirect::to("/auth/login"));
    }

    let jwt_secret = ctx.config.get_jwt_config()?;

    // let token = user
    //     .generate_jwt(&jwt_secret.secret, &jwt_secret.expiration)
    //     .or_else(|_| unauthorized("unauthorized!"))?;

    Ok(Redirect::to("/dashboard"))
}

pub fn routes() -> Routes {
    Routes::new()
        .add("/api/auth/register", post(register))
        .add("/api/auth/verify", post(verify))
        .add("/api/auth/login", post(login))
        .add("/api/auth/forgot", post(forgot))
        .add("/api/auth/reset", post(reset))
        .add("/api/auth/current", get(current))
        .add("/auth/register", get(register_view))
        .add("/auth/register", post(register_via_form))
        .add("/auth/login", get(login_view))
        .add("/auth/login", post(login_via_form))
}
