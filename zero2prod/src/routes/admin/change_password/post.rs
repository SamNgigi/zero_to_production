use actix_web::{HttpResponse, web};
use actix_web_flash_messages::FlashMessage;
use secrecy::{ExposeSecret, SecretString};
use sqlx::PgPool;

use crate::{
    authentication::{self, AuthError, Credentials, UserId, validate_credentials},
    routes::admin::dashboard::get_username,
    utils::{e500, see_other},
};

#[derive(serde::Deserialize)]
pub struct FormData {
    current_password: SecretString,
    new_password: SecretString,
    confirm_password: SecretString,
}

pub async fn change_password(
    db_pool: web::Data<PgPool>,
    form: web::Form<FormData>,
    user_id: web::ReqData<UserId>,
) -> Result<HttpResponse, actix_web::Error> {
    let user_id = *user_id.into_inner(); // NOTE: Dereferencing.

    if form.0.new_password.expose_secret() != form.0.confirm_password.expose_secret() {
        FlashMessage::error(
            "New Password and Confirm Password fields do not match. Fields must match.",
        )
        .send();
        return Ok(see_other("/admin/change_password"));
    };

    let username = get_username(&db_pool, user_id).await.map_err(e500)?;
    let credentials = Credentials {
        username,
        password: form.0.current_password,
    };
    if let Err(e) = validate_credentials(&db_pool, credentials).await {
        match e {
            AuthError::InvalidCredentials(_) => {
                FlashMessage::error("The current password is incorrect.").send();
                return Ok(see_other("/admin/change_password"));
            }
            AuthError::Unexpected(_) => {
                return Err(e500(e).into());
            }
        }
    };
    let new_password_len = form.0.new_password.expose_secret().len();
    if !(12..129).contains(&new_password_len) {
        FlashMessage::error("New password is too short. Password should be more than 12 but less than 129 characters long.").send();
        return Ok(see_other("/admin/change_password"));
    }

    authentication::update_password(&db_pool, form.0.new_password, user_id)
        .await
        .map_err(e500)?;
    FlashMessage::info("You've successfully changed your password.").send();
    Ok(see_other("/admin/change_password"))
}
