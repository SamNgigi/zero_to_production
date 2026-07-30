use actix_session::{Session, SessionExt, SessionGetError, SessionInsertError};
use actix_web::{FromRequest, HttpRequest, dev::Payload};
use std::future::{Ready, ready};
use uuid::Uuid;

pub struct TypedSession(Session);

impl TypedSession {
    const USER_ID_KEY: &'static str = "user_id";

    pub fn renew(&self) {
        self.0.renew()
    }

    pub fn insert_user_id(&self, user_id: Uuid) -> Result<(), SessionInsertError> {
        self.0.insert(Self::USER_ID_KEY, user_id)
    }

    pub fn get_user_id(&self) -> Result<Option<Uuid>, SessionGetError> {
        self.0.get::<Uuid>(Self::USER_ID_KEY)
    }
}

impl FromRequest for TypedSession {
    /* INFO:
     * This is a complicated way of saying
     * "We return the same error returned by the implementation of
     * `FromRequest` for actix_session `Session`"
     * */
    type Error = <Session as FromRequest>::Error;

    /* INFO:
     * Though Rust does now support async traits the actix_web `FromRequest` trait
     * has not ported their original implementation to use native async.
     * `from_request` expects a `Future` as a return type to allow for extractors
     * that need to perform async operations (e.g. a HTTP call)
     * In our case we aren't performing anything that requires async so we wrap
     * `TypedSession` into a `Ready` to convert it into a `Future` that resolves
     * to the wrapped value the first time it's pooled by the executor.
     * */
    type Future = Ready<Result<TypedSession, Self::Error>>;

    fn from_request(req: &HttpRequest, _payload: &mut Payload) -> Self::Future {
        ready(Ok(TypedSession(req.get_session())))
    }
}
