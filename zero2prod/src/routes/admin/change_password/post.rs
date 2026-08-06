use actix_web::{HttpResponse, web};
use actix_web_flash_messages::FlashMessage;
use secrecy::{ExposeSecret, SecretString};

use crate::{
    session_state::TypedSession,
    utils::{e500, see_other},
};

#[derive(serde::Deserialize)]
pub struct FormData {
    current_password: SecretString,
    new_password: SecretString,
    confirm_password: SecretString,
}

pub async fn change_password(
    form: web::Form<FormData>,
    session: TypedSession,
) -> Result<HttpResponse, actix_web::Error> {
    if session.get_user_id().map_err(e500)?.is_none() {
        return Ok(see_other("/login"));
    }
    if form.0.new_password.expose_secret() != form.0.confirm_password.expose_secret() {
        FlashMessage::error(
            "New Password and Confirm Password fields do not match. Fields must match.",
        )
        .send();
        return Ok(see_other("/admin/change_password"));
    };
    let _current_password = form.0.current_password.expose_secret();
    todo!()
}
