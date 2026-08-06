use actix_web::HttpResponse;

use crate::{
    session_state::TypedSession,
    utils::{e500, see_other},
};

pub async fn change_password_form(session: TypedSession) -> Result<HttpResponse, actix_web::Error> {
    todo!()
}
