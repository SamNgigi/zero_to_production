# 10. Securing Our API - B

##### 10.01.1.0. Skimming: What did you notice and why? Any Questions

_**What?**_  
Currently the `/newsletters` endpoint accepts a newsletter issue as JSON and then sends emails out to all subscribers.  

_**Why?**_  
Only priviledged uses should be able to publish a newsletter (create a newsletter issue) and send it out to subscribers.  
Right now anyone can hit the endpoint and broadcast whatever they want to our existing mailing list.

_**Questions?**_  
What is the best way to breakdown this chapter because it is quite a bulky one?  

While perusing the bulkiest sections are
- Password-based Authentication
- Login
- Sessions + Seed Users

So a session needs to ensure that we have enough context to clear a section without information overload.

## 10.06. Login. (_bulky_)

##### 10.06.0.0. Skimming: What did you notice and why? Any Questions

_**What?**_
- `login_form` which is a handler for `GET /login` and `login` which is a hander for`POST /login`.  
  They are defined in separate `src/routes/login/get.rs` and `src/routes/login/post.rs` modules
- [MDN Web Docs](https://developer.mozilla.org/en-US/docs/Web/HTTP/Guides/Redirections) on `3xx` redirects
- Adding an `AuthError` enum that we now use on `validate_credentials` as opposed to `PublishError`.
- Passing error information from `login` to `login_form` handers via HTML hard coded in the `login_form` handler
- `htmlescape` crate that helps us prevent XSS attacts (Cross-Site-Scripting Attacks)
- HMAC (**H**ash-based **M**essage **A**uthentication **C**odes): Keyed-Hasing for Message Authentication
  [RFC2104](https://datatracker.ietf.org/doc/html/rfc2104)
- `actix_web::error::InternalError` that we use with `LoginError` i.e `InternalError<LoginError>`
- Repository [snapshot](https://github.com/LukeMathWalker/zero-to-production/tree/root-chapter-10-part1) at the end of section _10.6.4.7 - Error Messages Must Be Ephemeral_
- Flash messages using cookies
- Finally adding `/login` tests when about to set error messages
- `post_login` adds a trait bound instead `Body: serde::Serialize` instead of using just a plain `body: serde_json:Value`
- Updating our `TestApp` struct to include `api_client: reqwest::Client` and refactoring all our calls to `reqwest::Client` to use `self.api_client` instead
- We update our `login_form` handler to use the `HttpReques` itself to retrieve the cookie instead of using the `Query` extractor with `QueryParams` struct we defined.
- `add_removal_cookie` method that automatically sets `_flash` cookie `MAX-AGE=0` behind the scenes.
- `actix-web-flash-messages` for setting, passing, and revoming cookie flash messages.
- Went into a rabbit hole and explored [axum-login](https://github.com/maxcountryman/axum-login)

_**Why?**_
- Was wondering why our login module had both a `get.rs` and `post.rs`. Now I understand.
- Interesting supplimentary material
- We to a `match` to convert an `AuthError` into a `PublishError`. Curious we could get a clean conversion using `#[source AuthError]` instead.
- This would be the place to explore `askama` or `tera` crate.
- First actix specific addition that we'll have to map to the axum equivalent
- Cool that the author acknowledges that the chapter is long.
- Looking forward to learning about cookies.
- Was thinking how one would go about testing the `/login` handlers if we didn't end up testing the implementation.
- We are introduced to another way of implementing a test helper.
- How we are able to persist the cookie store when using the `reqwest::Client` for client side interactions.
- `HttpRequest` allows us to access the cookie store.
- Like the pattern the chapter has followed around being explicit around the implementation and then showing the abstraction.
- Since we are using `actix-web-flash-messages` is there an axum equivalent.

_**Question?**_
- What is the axum equivalent of `actix_web::error::InternalError`?
- What is the axum equivalent of `actix-web-flash-messages`
  > Seems the choice is between
  > - [axum-flash](https://github.com/davidpdrsn/axum-flash) light weight
  > - [axum-messages](https://github.com/maxcountryman/axum-messages) full-fledged built ontop of tower sessions, modeled
  >   closely after Django Messages similar to `actix-web-flash-messages`.  
  >   Seems like the best drop in axum alternative for `actix-web-flash-messages
- Curious where to draw the line when it comes cookies, sessions, authorization/authentication between front end and purely API backends.
  > [Claude](https://claude.ai/chat/d40b81b5-4367-4ce2-aa32-0704572e87a6) chat where we explore this for a bit.


##### 10.06.0.1. Deep Dive: Summarize, ELI5, Connect

_**Summary**_  

In this section we primarily go through 
- serving the login form html
- redirecting appropriately when the login form data is submitted
- processing the login html form & adding an authentication module that will assist in doing this
- providing feedback to users incase of issues encountered while logging in.

### 10.06.0. Overview

##### 10.06.0.0.1. Deep Dive: Summarize, ELI5, Connect

_**Summary**_  

Here we add a `src/routes/login` module with `mod.rs`, `get.rs`, `post.rs` and `login.html`.  
We then stub the up appropriately then wire them up to `src/routes/mod.rs` and `startup.rs`

Let start by adding initial contents of `login.html`, `get.rs`, `post.rs`and `mod.rs`.
```HTML
<!--src/routes/login/login.html-->

<!DOCTYPE html>
<html lang="en">
  <head>
    <meta http-equiv="content-type" content-type="text/html" charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <link rel="icon" href="data:,">
    <title>Login</title>
  </head>
  <body>
    <form>
      <label>
        <input 
          type="text"
          placeholder="Enter Username"
          name="username">
      </label>
      <label>
        <input 
          type="password"
          placeholder="Enter Password"
          name="password">
      </label>
      <button type="submit">Login</button>
    </form>
  </body>
</html>
```

```Rust
//! src/outes/login/get.rs
use actix_web::{HttpResponse, http::header::ContentType};

pub async fn login_form() -> HttpResponse {
    HttpResponse::Ok()
        .content_type(ContentType::html())
        .body(include_str!("./login.html"))
}
```

The above is similar to what we did before when serving the `home.html` via the `home` handler in `src/routes/home/mod.rs`.  
Lets wire this up and this to `startup.rs`'s `.route`.

```Rust
//! src/routes/login/mod.rs
mod get;

pub use get::login_form;
```

```Rust
//! src/routes/mod.rs
// [...]
mod login;
// [...]

// [...]
pub use login::*;
// [...]
```

```Rust
//! src/startup.rs
use crate::route::{/**/, login_form};
// [...]

// [...]

fn run(/**/) -> Result<Server, std::io::Error> {
    // [...]
    let server = HttpServer::new(move || {
        App::new()
            // [...]
            .route("/login", web::get().to(login_form))
            // [...]
    });
    // [...]
}
```

Lets compile this and run this and look at what we have at `/login`

### 10.06.1. HTML Forms

![image.png](10_b_securing_our_api_files/c44ca008-d37b-45a3-b50d-2825a4fae4f4.png)

![image.png](10_b_securing_our_api_files/30485ade-7f92-4bb2-a34e-1f5bb40d7b6e.png)

The default `form` is to submit the data to the very same page it is being served from (i.e `/login`) using the `GET` verb. This is far from idea because as we can see forms we submit via `GET`  
encodes our input data in clear text as query parameters. Because query parameters are part of the URL they end up being part of the navigation history and are also captured by the logs

![image.png](10_b_securing_our_api_files/0fd92a70-3f43-42b5-864d-e754b1b14fc5.png)

To change this behavior we add `method` and `action` attribute to the `form` element as follows

```HTML
<!--src/routes/login/login.html-->
<!--[...]-->
    <form method="POST" action="/login">
<!--[...]-->
```

By adding `method="POST"` the input data becomes part of the request body posted to the `/login` endpoint, which is a much safer option.

![image.png](10_b_securing_our_api_files/1a108c99-567e-4ff5-9420-3f61d5b4f37c.png)

Our request now looks like the above image. We are getting `404 NOT FOUND` because we have not yet added a post `login` handler. Lets do so and define the endpoint.

### 10.06.2. Redirect On Success

```Rust
//! src/outes/login/login.rs
use actix_web::{HttpResponse, http::header::LOCATION};

pub async fn login() -> HttpResponse {
    HttpResponse::SeeOther()
        .insert_header((LOCATION, "/home"))
        .finish()
}
```

In this `login` handler we introduce `SeeOther` with is a [`303`](https://developer.mozilla.org/en-US/docs/Web/HTTP/Reference/Status/303) redirect which is the most fitting  
redirect for our usecase, because after a successful login, we want to redirect users to the `home` page. 
> [MDN Web Docs](https://developer.mozilla.org/en-US/docs/Web/HTTP/Guides/Redirections) provides a comprehensive guide on `3xx` range redirect status codes.

Lets wire this up
```Rust
//! src/routes/login/mod.rs

mod get;
mod login;

use get::login_form;
use post::login;
```

```Rust
//! src/startup.rs
use crate::route::{/**/, login};
// [...]

// [...]

fn run(/**/) -> Result<Server, std::io::Error> {
    // [...]
    let server = HttpServer::new(move || {
        App::new()
            // [...]
            .route("/login", web::post().to(login))
            // [...]
    });
    // [...]
}
```

The code should compile and now when we submit the form we are greeted with `"Welcome to our newsletter"` as shown below

![20260710-0740-55.0795612.gif](10_b_securing_our_api_files/36c43df5-6040-4717-abd7-a0b680598222.gif)

### 10.06.3.Processing Form Data

#### 10.06.3.0. Overview

##### 10.06.3.0.1 Deep Dive: Summarize, ELI5, Connect 

_**Summary**_  
Here we primarily review how we can best process the data submitted in the login page.
We will need a `FormData` struct to hold the `username` and `password` and the `Form` extractor provided by `actix_web` to get the submitted values.
```Rust
//! src/routes/login/login.rs
// [...]
use actix_web::web


#[derive(serde::Deserialize)]
struct FormData {
    username: String,
    password: SecretString,
}

pub async fn login(_form: web::Form<FormData>) -> HttpResponse {
    // [...]
}
```

From there we need to be able to validate the credentials. So we will need to make use of the `validate_credentials` However as it is right now 
[`validate_credentials`](https://github.com/SamNgigi/zero_to_production/blob/1e410b8df2855f4a366855fecdb484444711c338/zero2prod/src/routes/newsletters.rs#L141) is the `newsletter.rs` module returning a
`PublishError`   
incase of a failure mode. 

Validating credentials is ideally an authentication/authorization function. This means `validate_credentials` and the associated logic makes much more sense in an `authentication`  
module returning an `AuthErr`. We can then be able to use it across login functionality and else where where we might need to verify credentials.

That's what we do in the next sub-section.

#### 10.06.3.1. Building An `authentication` Module

In this sub-section we will 
1. Add an `authentication` module
2. Add `AuthError` enum for error handling.
3. Refactor and move `validate_credentials` and associated login `Credentials` struct, `get_stored_credentials` and `verify_password_hash` functions to
   the auth module
4. Update `publish_newsletter` accordingly.
4. Use `validate_credentials` in `login` handler to validate user credentials from the web form.
    - Add `LoginError` for error handling in the handler.
    - Map errors appropriately and/or redirect accordingly.

Our `authentication.rs` module will basically look like this;
```Rust
//! src/authentication.rs
use anyhow::Context;
use argon2::{Argon2, PasswordHash, PasswordVerifier}
use secrecy::SecretString;
use sqlx::PgPool;
use uuid::Uuid;

#[derive(thiserror::Error)]
pub enum AuthError {
    #[error("Invalid Credentials")]
    InvalidCredentials(#[source] anyhow::Error),
    #[error(transparent)]
    Unexpected(#[from] anyhow::Error)
}

pub struct Credentials {
    username: String,
    password: SecretString,
}

#[tracing::instrument(
    name = "Validate Credentials",
    skip(db_pool, credentials)
)]
pub async fn validate_credentials(
    db_pool: &PgPool,
    credentials: Credentials
) -> Result<Uuid, AuthError> {
    let mut user_id = None;
    let mut expected_password = SecretString::from(
        "$argon2id$v=19$m=19000,t=2,p=1$OqVpaPog6F9sxlWW5VoHkA$4uDo1cl2daKq1ZgmmvtQBfG3wwmI8Nk4i8gHk6pwrYA".to_string()
    );

    if let Some((stored_user_id, stored_expected_password)) = get_stored_credentials(db_pool, &credentials.username).await? {
        user_id = Some(stored_user_id);
        expected_password = stored_expected_password;
    };

    spawn_blocking_with_tracing(|| {
        verify_password_hash(expected_password, credentials.password)
    })
    .await
    .context("Failed to spawn blocking task thread.")
    .map_error(AuthError::Unexpected)??;

    user_id.ok_or_else(|| AuthError::InvalidCredentials(anyhow::anyhow!("Invalid Username.")))

    
}

async fn get_stored_credentials(db_pool: &PgPool, username: &str) -> Result<Option<(Uuid, SecretString)>, anyhow::Error> {
    let row = sqlx::query!(
        r#"
            SELECT user_id, password_hash
                FROM users
            WHERE username = $1
        "#,
        username
    )
    .fetch_optonal(db_pool)
    .await
    .context("Failed to execute query to retreive stored credentials")?
    .map(|r| (r.username, SecretString::from(r.password_hash)));

    Ok(row)
}


async fn verify_password_hash(
    expected_password: SecretString,
    password_candidate: SecretString,
) -> Result<(), AuthError> {
    let expected_password_phc_fmt = PasswordHash::new(expected_password.expose_secret())
        .context("Failed to parse password has to PHC string format.")?

    Argon2::default()
        .verify_password(
            password_candidate.expose_secret().as_bytes(),
            &expected_password_phc_fmt
        )
        .context("Invalid Password.")
        .map_err(AuthError::InvalidCredentials)
    
}
```

The main thing here is that we have moved `validate_credentials`, `get_stored_credentials` and `verify_password_hash` from `src/routes/newsletter.rs` to `src/authentication.rs`  
and now return `AuthError` now in the failure modes.  

With this we need to first update `src/routes/newsletters.rs` where  `validate_credentials` is being called, `publish_newsletter` and map propagated errors to appropriate
`PublishError`.
```Rust
//! src/routes/newsletter.rs
// [...]
use crate::authentication::{validate_credentials, AuthError, Credentials};

// [...]

#[tracing::instrument(/**/)]
pub async fn publish_newsletter(/**/) -> Result<HttpResponse, PublishError> {
   // [...] 
    let user_id = validate_credentials(/**/).await.map_err(|e| match e {
        AuthError::InvalidCredentials(_) => PublishError::Auth(e.into()),
        AuthError::Unexpected(_) => PublishError::Unexpected(e.into()),
    })?;
    // [...]
}
```

Finally we use our authentication module in our `login` handler in `src/routes/login/post.rs` to validate credentials built from `FormData` from our web form.  
We also add a `LoginError` handler to appropriately handle propagated errors, and implement the `ResponseError` trait on it so as to return the appropriate status code  
for a failure mode.

```Rust
//! src/routes/login/post.rs
// [...]
use actix_web::{ResponseError, http::{StatusCode, header::LOCATION}};
use sqlx::PgPool;
    
use crate::authentication::{validate_credentials, AuthError, Credentials};

#[derive(thiserror::Error, Debug)]
enum LoginError {
    #[error("Authentication Failed.")]
    Auth(#[source] anyhow::Error),
    #[error(transparent)]
    Unexpected(#[from] anyhow::Error)
}

impl ResponseError for LoginError {
    fn status_code(&self) -> StatusCode {
        match self {
            LoginError::Auth => StatusCode::UNAUTHORIZED,
            LoginError::Unexpected => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}

// [...]

// Adding tracing to the handler to log the username and the retrieved user_id
// incase of a successful authentication.
#[tracing::instrument(
    name = "Login",
    skip(form, db_pool),
    fields(username = tracing::field::Empty, user_id = tracing::field::Empty)
)]
pub async fn login(
    form: web::Form<FormData>,
    db_pool: web::Data<PgPool>, // We are now injecting PgPool to retreive stored credentials from db.
) -> Result<HttpResponse, LoginError> {
    let credentials = Credentials {
        username: form.0.username,
        password: form.0.password,
    };
    tracing::Span::current().record("username", tracing::field::display(&credentials.username));
    
    let user_id = validate_credentials(&db_pool, credentials).await.map_err(|e| match e {
        AuthError::InvalidCredentials(_) => LoginError::Auth(e.into()),
        AuthError::Unexpected(_) => LoginError::Unexpected(e.into()),
    })?;
    tracing::Span::current().record("user_id", tracing::field::display(&user_id));

    Ok(HttpResponse::SeeOther()
      .insert_header((LOCATION, "/home"))
      .finish)
}
```

The code should compile. Submission of the form triggers a page load, resulting in `"Authentication Failed"` being shown on screen. Much better than what we had before.
> The default implementation of the `error_response` provided by `actix_web`'s `ResponseError` trait populates the body using the `Display` representation
> of the error returned by the request handler.

![20260710-1135-13.1725375.gif](10_b_securing_our_api_files/ff343ba3-1880-4f2f-9f3e-3caf10257f89.gif)

### 10.06.4. Contextual Errors (_bulky_)

#### 10.06.4.0. Overview

_**Summary**_  
Could we do better than what we have above?  
In this sub-section we go through various options to improve user experience when authentication fails.
1. Naive Approach - rendering raw login html with errors included the `ResponseError`'s `error_response` implementation of `LoginError`
   Not the best because;
   - We now have 2 almost identical login html pages that we need to maintain
   - The use would be requested to confirm resubmission if authentication fails and they reload the page.
2. Query Parameters - we could pass back the authentication errors from `POST /login`'s `login` hander back to `GET /login`'s `login_form`
   via adding query params. This helps us address the 2 issues above
   - We redirect back to the `GET /login` allowing us to reuse the `login.html`.
   - Because we are using a `GET /login` no resubmission issue.

   But what about;
   - XSS attacks if someone tampers with our query parameters
3. We add `html-escape` to sanitize query params preventing potentials XSS attacks.  
   But how can we;
   - verify our own error query params to prevent malicious attackers from impersonating our own requests.
4. We add `hmac` to introduce **h**ash-based **m**essage **a**uthentication **c**odes allowing use to verify messages originating from our server.
   But;
   - error messages are meant to be short lived. When we encode them as part of the URL query parameters the become part of browser history. If one
     was attempting a fresh login but the uses a historical URL with the error query params they are working with a html error.
5. We can pass `hmac` tagged authentication error messages as cookies that are short-lived and ephimeral to be rendered in the html

#### 10.06.4.1. Naive Approach

_**Summary**_  
Here's what we have for the naive approach if in the `ResponseError`'s `error_response` implementation for `LoginError` we return a the `login_html` but 
now with the errors injected. 

```Rust
//! src/routes/login/post.rs
// [...]

// [...]

impl ResponseError for LoginError {
    fn error_response(&self) -> HttpResponse {
        HttpResponse::build(self.status_code())
            .content_type(ContentType::html())
            .build(format!(
                r#" <!DOCTYPE html>
<html lang="en">
  <head>
    <meta http-equiv="content-type" content-type="text/html" charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <link rel="icon" href="data:,">
    <title>Login</title>
  </head>
  <body>
    <p><i>{}</i></p>
    <form method="POST" action="/login">
      <label>
        <input 
          type="text"
          placeholder="Enter Username"
          autocomplete="on"
          name="username">
      </label>
      <label>
        <input 
          type="password"
          placeholder="Enter Password"
          name="password">
      </label>
      <button type="submit">Login</button>
    </form>
  </body>
</html> "#,
            self
        ))
    } 

    fn status_code(&self) -> StatusCode {
        match self {
            LoginError::AuthenticationFailed(_) => StatusCode::UNAUTHORIZED,
            LoginError::Unexpected(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}

// [...]
```

As we can see this is the same html as `login.html` just with the error passed in via `self`. Not quite scalable.

#### 10.06.4.2. Query Parameters

Alright, so we pass back the error message to the `GET /login`  endpoint as query parameters. How would we do that?
1. Do a redirect from the `error_response` back to the  `/login`.
2. Return the error message as query params when redirecting i.e. `format!("/login?error={}", the_error_msg)`
3. Extract the query params in `src/routes/login/get.rs` handler via `web::Query` extractor.

We need the `urlencoding` crate to appropriately encode our error messages into a URL format.
```bash
cargo add urlencoding
```
The we update our implementation as follows
```Rust
//! src/route/login/post.rs
// [...]
use urlencoding;

impl ResponseError for LoginError {
    fn error_response(&self) -> HttpResponse {
        HttpResponse::build(self.status_code())
            .insert_header((LOCATION, format!("/login?error={}", urlencoding::Encoded::new(self.to_string()))))
    }

    fn status_code() -> StatusCode {
        StatusCode::SEE_OTHER
    }
}
```

```Rust
//! src/route/login.get.rs
// [...]

#[derive(serde::Deserialize)]
struct QueryParams {
    error_msg: Option<String>,
}

async fn login_form(query: web::Query<QueryParam>) -> HttpResponse {
    let error_html = match query.0.error_msg {
        None => "".to_string(),
        Some(err_msg) = format!("<p>{}</p>", err_msg)
    }

    HttpResponse::Ok()
        .content_type(ContentType::html))
        .body(format!(include_str!("./login.html"), error_html = error_html))
}
```

![20260713-0712-16.1469524.gif](10_b_securing_our_api_files/773c9484-b172-4eb1-9d4e-d1ba9c2e8a4e.gif)

It works!

#### 10.06.4.3. Cross-Site Scripting (XSS) 

Query Params in the URL are not private, and nothing prevents a user or an attacker from playing with them to alter them to their purposes. For example try the link below.
```
http://localhost:8000/login?error=Your%20account%20has%20been%20locked%2C%20please%20submit%20your%20details%20%3Ca%20href%3D%22https%3A%2F%2Fzero2prod.com%22%3Ehere%3C%2Fa%3E%20to%20resolve%20the%20issue.
```
This is the result you get

![image.png](10_b_securing_our_api_files/51dc308d-c23b-48c2-8ac2-ae350d180cad.png)

Fortunately this only leads us to the books website, but an attacker could easily use this for a _phishing attack_, where a victim is lured to give up their credentials or click on malicous software.  
This is an example of [Cross Site Scripting (XSS) Attack](https://owasp.org/www-community/attacks/xss)

The OWASP provides a [cheatsheet](https://cheatsheetseries.owasp.org/cheatsheets/Cross_Site_Scripting_Prevention_Cheat_Sheet.html) of recommendations on how to prevent XSS attacks. Following their
guidelines we need to escape "code" characters that our application might inadvertently execute. 
For this we have the `html-escape` crate. Lets use it 
```bash
cargo add html-escape
```

> The book uses `htmlescape` crate which looks like its no longer actively maintained. We reach for `html-escape` and use its `encode_safe` which  
> is `htmlescape::escape_miminal`'s equivalent. From the documentation:
>
> <div style="background-color: #313B51; color: white; border-radius: 10px; padding: 10px; border: 1px solid white;">
>    
>  Encode text to prevent special characters functioning.
>  The following characters are escaped:
>   <ul>
>       <li><code>&</code> => <code>\&amp;</code></li>
>       <li><code><</code> => <code>\&lt;</code></li>
>       <li><code>></code> => <code>\&gt;</code></li>
>       <li><code>"</code> => <code>\&quot;</code></li>
>       <li><code>'</code> => <code>\&#x27;</code> </li>
>       <li><code>/</code> => <code>\&#x2F;</code> </li>
>   </ul>
> </div>
>

```Rust
//! src/routes/login/get.rs
// [...]
use html_escape;

// [...]

pub async fn login_form(/**/) -> HttpResponse {
    let error_html = query.0.error {
        None => "".to_string(),
        Some(err_msg) => format!("<p>{}</p>", html_escape::encode_safe(err_msg))
    }

    // [...]
}
```

Works!

![image.png](10_b_securing_our_api_files/8938a021-2036-4582-890e-4f4d84f19495.png)

However we need a more robust way of ensuring our messages and secure and verifiable by us. This is because nothing is stopping a attacker from changing our error message and adding their phone number or fake business contact info.

![image.png](10_b_securing_our_api_files/167817f4-ef71-4514-ba01-3d96e7cba63b.png)

#### 10.06.4.4. Message Authentication Codes

We need a way to verify that our error messages have been setup by our API and have not been altered any third party. This is known as **message authentication**. A way of guaranteeing that our  
messages have not be altered in transit (**integrity**) and that we can verify the guarentee of the sender (**data origin authentication**).

We achieve this using Hash-based Message Authentication Codes, we our message is tagged with a hash from our API.  
Here's a [good resource](https://www.youtube.com/watch?v=wlSG3pEiQdc) on [HMAC](https://en.wikipedia.org/wiki/HMAC).

Basically this generates a hash that we tag our messages with that can 
1. Verify the integrity of our message in that it has not been alterered. (_The message actually makes part of the final hash_)
2. Verify the identity of the sender because a secret key that only the sender has is part of the hash as well.

#### 10.06.4.5. Add An HMAC Tag To Protect Query Parameters

To do this we will need to
1. Add the `hmac` and `sha2` crates
2. Move our error redirection from `error_response` back to the `login_handler` matching on the result of `validate_credentials`
    - We need add `secret_key` to app state because we'll use it to generate our hmac 
    - Build out our hmac and pass it as query params. **Note** that we hash the whole `"error={}" query` and not just the populated error returned
    - Intial implementation pass returns a HttpResponse to highlight the fact that this approach disables us from loggin the errors
3. Update implementation to return a result with an `actix_web::error::InternalError` that wraps our Login error inorder to get our tracing back
4. Update `config.rs`, `startup.rs` `configuration/base.yaml` with a `secret_key` that we add to the application state to extract in `login_form`.
    - We add a `HmacSecret` wrapper type so that we avoid conflicting `SecretString` that might later be apart of the application state.

We add the `hmac` and `sha2` crates.
> Attempted to generate a `hmac` using `sha3::Sha3_256` however this returns a `does not implement CoreProxy trait`.
> Can look into it later but the immediate remedy is to default to `sha2::Sha256`
```bash
cargo add hmac sha2
cargo remove sha3
```

We could try add a hmac tag in our `impl ResponsError for LoginError` block. However we would have a challenge getting the `secret_key` required to construct  
the `hmac_tag` that will be used to verify
1. The integrity of our error message
2. The identity of the sender which is our API.

So we move back the login for setting our error parameters back to the main body of `login` and update our implementation as follows.  
```Rust
//! src/routes/login/post.rs
// [...]
use actix_web::error::InternalError;
use hmac::{Hmac, KeyInit, Mac};
use sha2::Sha256;
use secrecy::ExposeSecret;

use crate::startup::HMACSecret;

// [...]

#[tracing_instrument(
    name = "Login Credential Validation",
    skip(db_pool, credentials, secret_key),
    fields(username = tracing::field::Empty, user_id = tracing::fields::Empty)
)]
pub async fn login(
    db_pool: web::Data<PgPool>,
    credentials: web::Form<FormData>,
    secret_key: web::Data<HMACSecret>,
) -> Result<HttpResponse, InternalError<LoginError>> {
    // [...]
    
    match validate_credentials(&db_pool, credentials).await {
        Ok(user_id) => {
           tracing::Span::current().record("user_id", tracing::field::display(&usern_id));
            Ok( HttpResponse::SeeOther()
                .insert_header((LOCATION, "/home"))
                .finish() )
        }
        Err(e) => {
            let e = match e {
                AuthError::InvalidCredentials(_) => LoginError::AuthenticationFailed(e.into()),
                AuthError::Unexpected(_) => LoginError::Unexpected(e.into()),
            };
            let error_query = format!("error={}", urlencoding::Encoded::new(e.to_string()));
            let hmac_tag = {
                type Hmac256 = Hmac::<Sha256>;
                let mut mac = Hmac256::new_from_slice(secret_key.0.expose_secret().as_bytes())
                    .expect("Hmac can take key of any size");
                mac.update(error_query.as_bytes());
                mac.finalize().into_bytes()
            };

            let response = HttpResponse::SeeOther()
                .insert_header((
                    LOCATION,
                    format!("/login?{}&tag={}", error_query, hex::encode(hmac_tag))
                ))
                .finish();
            Err(InternalError::from_response(e, response))
        }
    }
    
}
```

Here we do the appropriate mappping to `LoginError` and then to be able to access the error in our logs with wrap in an `InternalError` through its `from_response` method that implements  
`ResponseError`. In `from_response` we pass an instance of `LoginError` (`e`) and the redirect response. We then wrap this in the `Err` variant.

Another thing to notice is that we have introduced a `HMACSecret` into the application state. We need the rest of our implementation to be updated accordingly, starting from   
1. `configuration/base.yaml`.
```yaml
#! configuration/base.yaml
application:
    # [...]
    secret_key: "long-and-very-secret-random-key-needed-to-verify-message-integrity"

# [...]
```
2. `src/config.rs`
```Rust
//! src/config.rs
// [...]

#[derive(serde::Deserialize, Clone)]
pub struct AppSettings {
    // [...]
    pub secret_key: SecretString,
}
```

3. `src/startup.rs`
```Rust
//! src/startup.rs
use secrecy::SecretString;

impl Application {
    pub async fn build(config: Settings) -> Result<Self, std::io::Error> {
        // [...]

        let server = run(
            // [...]
            config.appl.secret_key
        )?;

        // [...]
    }
    
    // [...]
}

// [...]

#[derive(slone)]
pub struct HMACSecret(pub SecretString);

fn run(
    // [...]
    secret_key: SecretString
) -> Result<Server, std::io::Error> {
    // [...]
    let secret_key = web::Data::new(HMACSecret(secret_key));

    let server = HttpServer::new( move || {
        App.new()
            // [...]
            .app_data(secret_key.clone())
    })
    // [...]

    // [...]
}
```

#### 10.06.4.6. Verifying The HMAC Tag

This is slighty more straight forward
1. We'll need to update our `QueryParam` to also include a `tag` now.
2. We add a `verify` method implementation for `QueryParams` that helps us validate our query params
2. We also need to update an optional `web::Query<QueryParams>` compared to having individual fields on `QueryParams` optional  
   This is because we either have both the error message and the tag or nothing at all, so that we are not trying to process invalid
   error only or tag only
3. We handle both cases for optional `QueryParams`
4. We handle both cases for the `Ok` and `Err` case of `verify`

```Rust
//! src/routes/login/get.rs
// [...]
use hmac::{Hmac, KeyInit, Mac};
use sha2:Sha256;
use secrecy::ExposeSecret;
use crate::startup::HMACSecret;

#[derive(serde::Deserialize)]
struct QueryParams {
    error: String,
    tag: String,
}

impl QueryParams {
    fn verify(self, secret_key: &HMACSecret) -> Result<String, anyhow::Error> {
        let tag = hex::decode(self.tag)?;
        let error_query = format!("error={}", urlencoding::Encoded::new(&self.error));

        type Hmac256 = Hmac<Sha256>;
        let mut mac = Hmac256::new_from_slice(secret_key.0.expose_secret().as_bytes())
            .expect("HMAC can take key of any size");
        mac.update(error_query.as_bytes());
        mac.verify_slice(&tag)?;
        
        Ok(self.error)
    }
}

#[tracing::instrument(
    name = "Login Form",
    skip(query, secret_key),
)]
pub async fn login_form(
    query: Option<web::Query<QueryParams>>,
    secret_key: web::Data<HMACSecret>,
) -> HttpResponse {
    let error_html = match query {
        None => "".to_string(),
        Some(error_query) =>  error_query.0.verify(&secret_key) {
            Ok(err_msg) => format!("<p>{}</p>", html_escape::encode_safe(err_msg)),
            Err(e) => {
                tracing::warn!(
                    error.message = %e,
                    error.cause_chain = ?e,
                    "Failed to verify error query parames using HMAC tag"
                );
               "".to_string() 
            }
        }
    };

    HttpResponse::Ok()
        .content_type(ContentType::html())
        .body(format!(include_str!("./login.html"), error_html))
}
```

When we compile and run this, when our authentication fails we get error messages that are tagged with our hmac, which we can use to verify.

![20260715-0655-21.8791896.gif](10_b_securing_our_api_files/8fb2b4ca-1048-4040-8ff6-7c6f1a275842.gif)

And messages that are tampared with would not be displayed.

![20260715-0708-55.9006862.gif](10_b_securing_our_api_files/328b9ab7-6612-4ef8-8329-511c623e0942.gif)

#### 10.06.4.7. Error Messages Mush Be Ephemeral.

However there are a couple of challenges with using query params for displaying our authentication error messages. For one, because query params are part of URL history,  
if someone were to refresh the page, they would be still have the now obsolete errors. This is undesirable.

What we want is for errors to be **ephimeral** meaning that they are displayed only on a single failed authentication attempt and does not become part of the browser history.
To get the error message again means a new failed authentication attempt.

How can we achieve this?  
**Cookies.**

#### 10.06.4.8. What Is A Cookie?

According to [MDN Web Docs](https://developer.mozilla.org/en-US/docs/Glossary/Cookie#:~:text=A%20cookie%20is%20a%20small%20piece%20of%20information%20left%20on%20a%20visitor%27s%20computer%20by%20a%20website%2C%20via%20a%20web%20browser.)  

<div style="background-color: #313B51; color: white; padding: 15px; border-radius: 15px; border: 1px solid white;">

A <strong>cookie</strong> is a small piece of data that a server sends to the user's web browser. The browser my store the cookie and send it back to the server with later requests.
    
</div>



The flow remains the same as we did with query params.
1. A use enteres invalid credentials and submits the form
2. `POST /login` handler `login` sets a cookie with the error message and then redirect to `GET /login`
3. The `GET /login` request now includes the error messages in the cookies currently set for the user.
4. The `GET /login` handler `login_form` checks for any error messages set in the cookie
5. The `GET /login` handler renders the html form with the error messages then deletes the content of the cookie.

The last step ensures that the error message are truly ephimeral, not forming part of the URL history and immediately disposed after use.  
This technique is know as **flash messages**.

#### 10.06.4.9. An Integration Test For Login Failures (_bulky_)

_**Summary**_  
As we are now entering the final iteration of our design, lets capture the desired behavior in a test.

In this section we'll
1. We'll add a `post_login` helper to `TestApp`.
2. Add `tests/api/login` module
3. Add `an_error_flash_message_cookie_is_set_on_failure()` test and implement first iteration whereby
   - We test that the redirection occurs and returns a `303` status
   - This test should fail with a `200` vs `303` mismatch, leading us to update the `post_login` with a no redirect policy.
   - This should also fail but now because we still the error query params added by the `login` handler. We remove the query params from the implementation.
   - We add `assert_on_redirect` helper function to group common redirect assertions that we will resuse
4. Update the test to extract cookies from the response and do first iteration of this that does not use the cookie api-(commit work)
5. Update the cookie extraction logic to make use of `reqwest`'s cookie api.

_**Implementation**_  

**1. Add `post_login`** helper to `TestApp`.
```Rust
//! tests/api/helpers.rs
// [...]

// [...]

impl TestApp {

    pub async fn post_login<Body>(&self, body: &Body) -> reqwest::Response 
    where Body: serde::Serialize
    {
        reqwest::Client::new()
            .post(format!("{}/login", &self.address))
            .form(body)
            .send()
            .await
            .expect("Fail to execute login POST request in test.")
    }

    // [...]
}
```
**Note:** because we use `.form` when buiding the `Client` we need to add it as part of the `reqwest` features
```bash
cargo add reqwests -F form
```

**2. Add `tests/api/login.rs` and 3. Add initial implementation of `an_error_flash_cookie_message_is_set_on_failure`**. 
```Rust
//! tests/api/login.rs <- NEW MODULE
use crate::helpers::spawn_app;

#[tokio::test]
async fn an_error_flash_message_cookie_is_set_on_failure() {
    // Arrange
    let app = spawn_app().await;
    let login_body = serde_json::json!({
        "username": "random-username",
        "password": "random-password",
    });
    // Act
    let response = app.post_login(&login_body).await;
    
    // Assert
    assert_eq!(response.status().as_u16(), 303);
}
```

The test fails with the following;

![image.png](10_b_securing_our_api_files/d9d05426-68a8-4d7e-9f62-15bec8488829.png)

Why are we getting a `200` in our response yet we know for a fact that we are redirecting?  
According to `reqwest`'s [documentation](https://docs.rs/reqwest/latest/reqwest/redirect/index.html#:~:text=Redirect%20Handling-,By%20default%2C%20a,.,-Structs)

<div style="background-color: #313B51; color: white; padding: 15px; border-radius: 15px; border: 1px solid white;">
    
By default a `Client` will automatically handle HTTP redirects, having a maximum redirect chain of 10 hops. To customize this
behavior, a `redirect::Policy` can be used with a `ClientBuilder`.
    
</div>

We will follow what the documentation [suggests](https://docs.rs/reqwest/latest/reqwest/redirect/struct.Policy.html#:~:text=none()%20%2D%3E%20Self-,Create%20a,that%20does%20not%20follow%20any%20redirect.,-Source) and update our implementation as follows  
```Rust
//! test/api/helpers.rs
// [...]

// [...]

impl TestApp {
    async fn post_login() -> reqwest::Response {
        reqwest::Client::builder()
            .redirect(reqwest::redirect::Policy::none())
            .build()
            .unwrap()
            // [...]
    }
    // [...]
}
```
The test should now pass.

Let's go further and inspect the `Location` header.  
We add `assert_or_redirect` helper function that will check both the redirect status and location header.
```Rust
//! tests/api/login.rs
// [...]

fn assert_on_redirect(response: &reqwest::Response, location: &str) {
    assert_eq!(response.status().as_u16(), 303);
    assert_eq!(response.headers().get("Location").expect("Failed to get Location header in test"), location);
}

#[tokio::test]
async fn an_error_flash_message_cookie_is_set_on_failure(){
    //[...]
    assert_on_redirect(response, "/login")
}
```

Now this fails with;

![image.png](10_b_securing_our_api_files/a2f2ac7c-1e4a-4767-9f8e-55365e5f2972.png)

Our `POST /login` handler `login` is still passing the error and hmac tag as query parameters.  
Since our goal is to now use cookies for flash error messages, we remove the setting of errors and hmac as query params from the `login` handler. 
```Rust
//! src/routes/login/post.rs
//[...]

// [...]

#[tracing::instrument(/**/)]
pub async fn login(
    db_pool: web::Data<PgPool>,
    query: web::Form<FormData>
    // Removed secret key params
) -> Result<HttpResponse, InternalError<LoginError>> {
    // [...]
    match validate_credentials(/**/).await {
        Ok(user_id) => {/**/},
        Err(e) => {
            let e = match e {
                AuthError::InvalidCredentials(_) => LoginEror::AuthenticationFailed(e.into()),
                AuthError::Unexpected(_) => LoginError::Unexpected(e.into()),
            };
            // Removed building the error and hmac query params
            let response = HttpResponse::SeeOther()
                .insert_header((LOCATON, "/login"))
                .finish();
            Err(InternalError::from_response(e, response))
        }
    }
}
```
The test should now pass.

**4. First iteration of extracting cookies in test.**  
To understand how to extract cookies we need to understand a little around co
How are cookies set?  
Cookies are set by attaching a special HTTP header to the response. In its simplest for this would look like.  
```HTTP
Set-Cookie: {cookie-name}={cookie-value}`
```
We can set cookies multiple times and attach them to a response using the `Set-Cookie` header once for each cookie.  
`reqwest` provides a `get_all` method to deal with multi-value cookie headers. With this we can go ahead an extract the cookies from our response in test,  
and assert the value we want back.
```Rust
//! tests/api/login.rs
// [...]
use reqwest::header::HeaderValue;
use std::collection::HashSet;

#[tokio::test]
async fn an_error_flash_message_cookie_is_set_on_failure() {
    // [...]
    let cookies: HashSet<_> =  response
        .header()
        .get_all("Set-Cookie")
        .into_iter()
        .collect();

    assert!(cookes.contains(&HeaderValuve::from_str("_flash=Authentication Failed.")));
}
```
Our test fails because we are not yet setting the cookie in our implementation.  
Before we do there is a more ergonomic way for us to check for the existence of our flash message cookie through a dedicated API that `reqwests` gives us.  
To use it we need to enable the feature
```bash
cargo add reqwests -F cookies
```
And then our implementation can be updated as follows

```Rust
//! tests/api/login.rs
// [...]
// No need for HeaderValue and Hashset now

#[tokio::test]
async fn an_error_flash_message_cookie_is_set_on_failure() {
    // [...]

    let flash_message = response.cookies().find(|c| c.name() == "_flash").expect("Failed to get cookie by provided name");
    assert_eq!(flash_message.value(), "Authentication Failed.");
}
```

The error still fails, though we now know how to use the underlying implementation that the `.cookiles()` abstracts away.

![image.png](10_b_securing_our_api_files/f979bb88-f1a0-499f-a684-2a40d4c828bc.png)

#### 10.06.4.10. How To Set A Cookie in `actix-web`

We can set the cookie directly by working with the headers as follows
```Rust
//! src/routes/login/post.rs
//[...]

#[tracing::instrument(/**/)]
pub async fn login(/**/) -> Result<HttpResponse, InternalError<LoginError>> {
    // [...]
    match validate_credentials(/**/).await {
        Ok(/**/) => {/**/}
        Err(e) => {
            let e = match e {
                AuthError::InvalidCredentials(_) => LoginError::AuthenticationFailed(e.into()),
                AuthError::Unexpected(_) => LoginError::Unexpected(e.into()),
            };

            let response = HttpResponse::SeeOther()
                .insert_header((LOCATION, "/login"))
                .insert_header(("Set-Cookie", format!("_flash={e}")))
                .finish();
            Err(InternalError::from_response(e, response))
        }
    }
}
```

`actix-web` does provide a cookies api that we can use.
```Rust
//! src/routes/login/post.rs
//[...]
use actix_web::cookie::Cookie;

#[tracing::instrument(/**/)]
pub async fn login(/**/) -> Result<HttpResponse, InternalError<LoginError>> {
    // [...]
    match validate_credentials(/**/).await {
        Ok(/**/) => {/**/}
        Err(e) => {
            let e = match e {
                AuthError::InvalidCredentials(_) => LoginError::AuthenticationFailed(e.into()),
                AuthError::Unexpected(_) => LoginError::Unexpected(e.into()),
            };

            let response = HttpResponse::SeeOther()
                .insert_header((LOCATION, "/login"))
                .cookie(Cookie::new("_flash", e.to_string()))
                .finish();
            
            Err(InternalError::from_response(e, response))
        }
    }
}
```

The test should go green.

#### 10.06.4.11. An Integration Test For Login Failures - Part 2

Alright we have the setting part down. We now need to test that the error message is display when we are redirected back to `GET /login`.  
We'll need to do the following
1. Add `get_login_html` test helper that returns a `String`
2. Call it in our login test and check that the returned html string contains the error message `"Authorization Failed."`

**1. Add `get_login_html`** test helper.
```Rust
//! test/api/helpers.rs
// [...]

// [...]

impl TestApp {
    pub async fn get_login_html(&self) -> String {
        reqwest::Client::new()
            .get(format!("{}/login", &self.address))
            .send()
            .await
            .expect("Failed to execute GET login page request in test.")
            .text()
            .await
            .expect("Failed to retreive login html as text in test.")
    }

    // [...]
}
```

**2. Use the `get_login_html` helper in our `an_error_flash_message_cookie_is_set_on_failure` to check error message is rendered on page.**
```Rust
//! test/api/login.rs
// [...]

#[tokio::test]
async fn an_error_flash_message_cookie_is_set_on_failure() {
    // Arrange
    // [...]
    
    // Act & Assert 1
    // [...]
    
    // Act & Assert 2
    let login_html = app.get_login_html().await;
    assert!(login_html.contains(r#"<p><i>Authorization Failed.</i></p>"#));
    
}
```

This test should fail because we are not reading our cookie message to render it in the `GET /login` html.

![image.png](10_b_securing_our_api_files/ca7353e5-b41c-4e7f-b23b-68d381ad7bee.png)

#### 10.06.4.12. How To Read A Cookie In `actix-web`

Here we update the `GET /login` handler `login_form` to extract the error flash message from the cookie. We remove our earlier implementation around  
`QueryParams`.

Our current implementation looks like [this](https://github.com/SamNgigi/zero_to_production/blob/6fe171a004e0721c5febee1ffb5926c03f362ba0/zero2prod/src/routes/login/get.rs).  
We are still extracting `QueryParams` and verify the error message with `HMACSecret`. We can go ahead and remove this, read and render the error message from the cookes instead.  
How do we do this?  

By working with `actix_web::HttpRequest` we can use `actix_web`'s cookie API to extract any flash messages that are part of the request. So our `login` handler simplyfies to this.
```Rust
//! src/routes/login.rs
// [...]
use actix_web::HttpRequest;

#[tracing::instument(/**/)]
pub async fn login_html(request: HttpRequest) -> HttpResponse {
    let error_html = match request.cookie("_flash") {
        None => "".into(),
        Some(e) = format!("<p><i>{}</i></p>", e.into()),
    };
    // [...]
}
```

Out test still fails. Why?  
We if we take a look at our `get_login_html` helper, we are using a new instance of a `reqwest::Client`, therefore we cannot propagate our cookie accross
`post_login` to `get_login_html`. To fix this we need to
1. Initialize a shared `client` which is a `request::Client` as part of our `TestApp`. This means we add a `client` field to `TestApp`.
2. Enable cookie storage in our `reqwest::Client`

To do so we update our `tests/api/helper.rs` as follows.
```Rust
//! tests/api/helpers.rs
//[...]

// [...]

pub struct TestApp {
    // [...]
    // New field
    pub client: reqwest::Client;
}

impl TestApp {
    pub async fn get_login_html(/**/) -> String {
        self.client
            .get(/**/)
            // [...]
    }
    
    pub async fn post_login(/**/) -> reqwest::Response {
        self.client
            .post(/**/)
            // [...]
    }
    
    pub async fn post_newsletter(/**/) -> reqwest::Response {
        self.client
            .post(/**/)
            // [...]
    }
    
    pub async fn post_subscription(/**/) -> reqwest::Response   {
        self.client
            .post(/**/)
            // [...]
    }
    
}

pub async fn spawn_app() -> TestApp {
    // [...]
    let client = reqwest::Client::builder()
        .redirect(reqwest::redirect::Policy::none())
        .cooke_store(true)
        .build()
        .expect("Failed to build reqwest::Client in test");
        
    // [...]

    let test_app = TestApp {
        // [...]
        client
    };

    // [...]
}
```

The test should now pass.

#### 10.06.4.13. How To Delete A Cookie In `actix-web`

We want to make sure that the error message is truly ephimeral such that when we do a fresh login, we don't get the login errors. To ensure this  
behavior we;
1. Update our test to trigger and `GET /login` login form
2. We check that we don't have authentication errors
3. We set our cookie to have a max age of zero then use the more ergonomic `add_removed

**1. Update our test to trigger `GET /login` again & 2. Check that cookie is ephimeral**  
In other words try triggering a fresh login and check that we don't have the `"Authentication Failed."`.  
How do we do this? We just call `get_login_html` again but this time we check that the error cooke does not exist.
```Rust
//! tests/api/login.rs
// [...]

async fn an_error_flash_message_cookie_is_set_on_failure() {
    // Arrange
    // [...]
    
    // Act & Assert 1
    // [...]
    
    // Act and Assert 2
    let login_html = app.get_login_html().await;
    assert!(
        login_html.contains(r#"<p><i>Authentication Failed.</i></p>"#),
        "Error Html Should Be Rendered."
    );

    // Act and Assert 3
    let login_html = app.get_login_html().await;
    assert!(
        !login_html.contains(r#"<p><i>Authentication Failed.</i></p>"#),
        "Error Html Should NOT Be Rendered."
    );

}
```

Our test should now fail.

![image.png](10_b_securing_our_api_files/c4712b0c-c47e-4b86-89ff-5edf9f3bec20.png)

**3. We set our cookie to have a max age of zero the use more ergonomic API.**.  
To enforce that our error flash message is actually _ephimeral_ we have to define how long our cookie should exist. To specify an expiration policy either as
1. `Max-Age` i.e. time to live in terms of seconds. e.g `Set-Cookie: _flash=omg; Max-Age=5;`
2. `Expire` - i.e. a date`Set-Cookie: _flash=omg; Expires=Thu, 7 July 2026 23:59:59 GMT;`

Setting `Max-Age` to 0 instruct the browser to immediately expire the cookie, which is what we want. To do this we can either set it via 
1. Calling `max_age` on a `Cookie` when constructing the  `HttpResponse`
```Rust
//! src/routes/login/get.rs
// [...]
use actix_web::cookie::{Cookie, time::Duration};

#[tracing::instrument(/**/)]
pub async fn login_form(request: HttpRequest) -> HttpResponse {
    let error_html = match request.cookie("_flash") {
        None => "".into(),
        Some(cookie) => format!("<p><i>{}</i></p>", cookie.value()), 
    };

    HttpResponse::Ok()
        .insert_header(ContentType::html())
        .cookie(Cookie::build("_flash", "").max_age(Duration::ZERO).finish())
        .body(format!(include_str!("./login.html"), error_html))
}

```
2. Calling `add_removal_cookie` on the `HttpResponse`

```Rust
//! src/routes/login/get.rs
// [...]
use actix_web::cookie::Cookie;

#[tracing::instrument(/**/)]
pub async fn login_form(request: HttpRequest) -> HttpResponse {
    let error_html = match request.cookie("_flash") {
        None => "".into(),
        Some(cookie) => format!("<p><i>{}</i></p>", cookie.value()), 
    };

    let mut response = HttpResponse::Ok()
        .content_type(ContentType)
        .body(format!(include_str!("./login.html"), error_html));

    response.add_removal_cookie(&Content::new("_flash", ""))
        .expect("Failed due to malformed name in cookie header");
    
    response
}
```


#### 10.06.4.14. Cookie Security

_**Summary**_.  
What kind of attacks can be mounted against cookies?  

While cookies present a reduced attack surface to XSS attacks compared to query parameters,
> constructing a clickable link that a naive user can just click and execute malicious code is a little more difficult here.

We still want to ensure that malicious actors cannot
- _tamper_ with our cookies thus compromising cookie content integrity.
- _sniff_ our coookie content, compromising the confidentiality.

A must have first line of defense is our request are over a secure encrypted connection (HTTPS) ensuring that communication between server and  
client cannot be intercepted, read or arbitrarily modified. Marking cookies as `Secure` ensures that the browser only attaches cookies to requests  
that are sent over secure HTTPS connections.

Second we want to ensure that JavaScript cannot, read and/or [overwrite our cookies](https://www.youtube.com/watch?v=U1DT0Ekswto). Marking cookies as `HTTP-Only` ensures that our cookies are not  
visible to JavaScript on the  browser cannot see our cookies to modify them.

Lastly cookies are visible to via the Developer tools. Nothing stops a user from freely manipulating their cookies.

Using HMAC to verify our cookie integrity and origin as we did with our query params remains the appropriate and robust solution to ensure 
our cookies authenticity. Instead of doing the cookie-hmac wiring manually, we lean on the [`actix-web-flash-messages`](https://crates.io/crates/actix-web-flash-messages) crate that makes things
easier for us. Because we already understand how `HMAC`s work and their purpose.

#### 10.06.4.15. `actix-web-flash-messages`

Primarily use `actix_web_flash_message` to manage cookie flash message by
- Registering its `FlashMessageFramework` as middleware in our `actix_web` App
- Adding the appropriate storage for our flash messages which in this case is `CookieMessageStorage`. 
- `CookieMessageStorage` requires that our messages be signed threfore we have to build it with a `Key` that takes our `secret_key`
- Now use `FlashMessage::error` to attach our error message as a flash message cookie that we can send as part of our request.
- Retriving our error message using `IncomingFlashMessages` and filtering according to the appropriate level to populate our `error_html`.

It everything including setting the right properties (`Scecure`, `Http-Only`), setting the appropriate default expiration policy & Hmac wiring.

Let start by wiring our flash messages middleware by completing the first 3 tasks above.
```Rust
//! src/startup.rs
// [...]
use actix_web_flash_messages::{FlashMessagesFramework, storage::CookieMessageStorage};
use actix_web::cookie::Key;
use secrecy::ExposeSecret;

// [...]

// NOTE: We removed the HMACSecret wrapper type.

fn run(/**/) -> Result<Server, std::io::Error> {
    // [...]
    let cookie_storage = CookieMessageStorage::builder(
        Key::from(secret_key.expose_secret().as_bytes())
    ).build(); // Building our cookie storage with Key
    let flash_messages = FlashMessagesFramework::builder(cookie_storage).build(); // Adding cookie_storage as our flash_messages storage backend
    let server = HttpServer::new(move || {
        App::new()
            .wrap(flash_messages.clone()) // registering the flash messages as middleware
            .wrap(TracingLogger::default())
            // [...]
    })
}
```

Then in our `POST /login` handler `login` we use `FlashMessage` to send over our error cookies.
```Rust
//! src/routes/login/post.rs
// [...]
use actix_web_flash_messages::FlashMessages;

#[tracing::instrument(/**/)]
pub async fn login(/**/) -> Result<HttpResponse, InternalError<LoginError>> {
    // [...]
    match validate_credentials(/**/) {
        Ok(/**/) => {/**/}
        Err(e) =>  {
            let e = match e {
                AuthError::InvalidCredentials(_) => LoginError::AuthenticationFailed(e.into()),
                AuthError::Unexpected(_) => LoginError::Unexpected(e.into()),
            };
            // Sending our error flash with all the appropriate properties set under the hood.
            FlashMessages::error(e.to_string()).send(); 
            
            let response = HttpResponse::SeeOther()
                .insert_header((LOCATION, "/login"))
                // No setting cookies here now.
                .finish();
            Err(InternalError::from_response(e, response));
        } 
    }
}
```

In our `GET /login` handler `login_form` we now check if we have any error flash message cookie that we can render in the login html.
```Rust
//! src/routes/login/get.rs
// [...]
use actix_web_flash_messages::{IncomingFlashMessages, Level};
use std::fmt::Write;

pub async fn login_form(flash_messages: IncomingFlashMessages) -> HttpResponse {
    let mut error_html = String::new();
    for msg in flash_messages.iter().filter(|msg| msg.level() == Level::Error) {
        writeln!(error_html, "<p><i>{}</i></p>", msg.content()).expected("Failed to write error flash message to error_html.");
    }
    HttpResponse::Ok()
        .content_type(ContentType::html())
        // No cookie removal
        .body(format!(include_str!("./login.html"), error_html))
}
```

Our tests should remain passing. But it seems there's one failing

![image.png](10_b_securing_our_api_files/d6c8a0d1-cf64-4989-8141-b0a323b45bdb.png)

Right. The one where we were checking equality of the error message contained in the cookie


```Rust
//! tests/api/login.rs
// [...]
// No need for HeaderValue and Hashset now

#[tokio::test]
async fn an_error_flash_message_cookie_is_set_on_failure() {
    // [...]
    
    // Act and Assert 1
    // [...]
    let flash_message = response.cookies().find(|c| c.name() == "_flash").expect("Failed to get cookie by provided name");
    assert_eq!(flash_message.value(), "Authentication Failed.");

    // Act and Assert 2
    // [...]
    
    // Act and Assert 3
    // [...]

}
```

That assertion now sits too close to the implementation details. We can go ahead and remove it.
```Rust
//! tests/api/login.rs
// [...]
// No need for HeaderValue and Hashset now

#[tokio::test]
async fn an_error_flash_message_cookie_is_set_on_failure() {
    // [...]
    
    // Act and Assert 1
    // No assertion on equality of cookie error message
    assert_on_redirect(&response, "/login");
    
    // Act and Assert 2
    // [...]
    
    // Act and Assert 3
    // [...]

}
```

## 10.07. Sessions.

### 10.07.0. Overview

##### 10.07.0.0. Skimming: What did you notice and why? Any Questions


_**What?**_  
- Sessions allow us not to have to re-authenticate a user everytime they want to interact with functionality they are already entitled to.
- **user** session vs **browser** session and how to tell apart which session we are talking about when talking about a _session cookie/session token_ lifetime. 
- Using Redis as our session storage as opposed to Postgres
- We are using `actix-session` for session management in actix
- Getting a Prod Redis URL on fly.io
- The `fn e500` error handling approach in `src/routes/admin/dashboard.rs`
- [Session Fixation Attack](https://acrossecurity.com/papers/session_fixation.pdf)
- Replacing the use of `actix-session::Session` with our custom type `TypedSession` with `fn from_request`.

_**Why?**_
- More clear on the purpose of sessions.
- Might be easily tripped up by what context we are taking about.
- Looking forward to how work with Redis in actix and axum
- We will likely use `tower-session` for managing session in axum
- Probably a good idea to get this url first thing tomorrow morning
  > Why tomorrow?
  > - Mind a little fresh to debug issues.
  > - Claude is also fresh in the early mornings.
- How we call `e500` in
  ```Rust
  let _username = if let Some(user_id) = session.get::<Uuid>("user_id").map_err(e500)? {
      todo!()
  } else {
      todo!()
  }
  ```
- Yet another cool resource on web application security. I'm guessing _Session fixation_ a session token is left disambiguated between anonymous and priviledged sessions.
- We get to implement async traits that require to explicity return a future. Looking forward to this as well.

  
_**Question?**_  
_Diff_ between a user session and a browser session.
 

| Feature | **User** | **Browser Session** |
| :--- | :--- | :--- |
| **Definition** | A single unique visitor. | A single continuous visit. |
| **Duration** | Long-term (persists across days/weeks). | Short-term (expires via inactivity or closure). |
| **Relationship** | **One user** can have **many sessions**. | **One session** belongs to exactly **one user**. |
| **Identifier** | Client-side cookie ID or account User ID. | Server/Client-generated Session ID. |
| **Storage Location** | Client browser cookie or server database. | RAM, Redis, or short-lived session cookie. |
| **Cookie Attributes** | `Expires` set to months or years in future. | `Expires` set to end when browser closes. |
| **State Management** | Stores persistent profile and account preferences. | Stores temporary data like shopping cart items. |


##### 10.07.0.0.1. Deep Dive: Summarize, ELI5, Connect



### 10.07.1. Session-based Authentication



### 10.07.2. Session Store

### 10.07.3. Choosing A Session Store

#### 10.7.3.0. Overview



#### 10.7.3.1. Postgres



#### 10.7.3.2. Redis



### 10.07.4. `actix-session`



### 10.07.5. Admin Dashboard

#### 10.7.5.0. Overview



#### 10.7.5.1. Redirect On Login Success



#### 10.7.5.2. Sessions



#### 10.7.5.3. A Type Interface To Sessions



#### 10.7.5.4. Reject Unauthenticated Users



## 10.08. Seed Users.

### 10.08.0. Overview

##### 10.08.0.0. Skimming: What did you notice and why? Any Questions

_**What?**_  
- The exercise to implement login protected functionality to invite more collaborators that inspiration for the subscription flow.
- Using `dbg!()` to get test PHC string for password.
- Added a utils module for `e500` and `see_otherr` functions.
- Creating a custom actix middle ware that wraps all restricted endpoint requests
- Challenge to implement _new password is too short_ ensuring new password has to be between 12-129 characters.


_**Why?**_  
- Seems like the invlte collaborators functionality would involve querying specific users from the db and sending them an invite link that is login protected first.
- Hacky but needed. Wonder how Django does it.
- Still unclear about how `e500` will work in axum.


_**Questions?**_  
None.


##### 10.08.0.0.1. Deep Dive: Summarize, ELI5, Connect



### 10.08.1. Database Migration



### 10.08.2. Password Reset. (_Slightly bulky_)

#### 10.8.2.1. Form Skeleton



#### 10.8.2.2. Unhappy Path: New Passwords Do Not Match



#### 10.8.2.3. Unhappy Path: The Current Password Is Invalid



#### 10.8.2.4. Unhappy Path: The New Password Is Too Short



#### 10.8.2.5. Logout



#### 10.8.2.6. Happy Path: The Password Was Changed Successfully



## 10.09. Refactoring.

### 10.09.0 Overview.

##### 10.09.0.0. Skimming: What did you notice and why? Any Questions

_**What?**_  
- `actix-web-lab` for writing an actix-web middleware
- Seems like we eventually write the middleware `reject_anonymous_users` in the end.
- Introducing a `web::scope` to wrap all `/admin` routes with projection from anonymous users.
- There's mention on an indemptency test at the end of the Refactoring section which I'm not able to get a lock on.

_**Why?**_  
- Seems like an iteresting challenge (Creating a custom middleware) in both actix and axum.
- Is there an axum equivalent for `actix-web-lab`
- Which indempotency check?

_**Question?**_  
- Which indempotency test?


##### 10.09.0.0.1. Deep Dive: Summarize, ELI5, Connect



### 10.09.1. How To Write An `actix-web` middleware



## 10.10. Summary.

##### 10.10.0.0. Skimming: What did you notice and why? Any Questions

_**What?**_  
- Cool set of challenges.

_**Why?**_  
- Hope by the end of the chapter I'm confident to tackle them head on without reference. Will let you know.

_**Question?**_  
None


##### 10.10.0.0.1. Deep Dive: Summarize, ELI5, Connect

To remember when deploying

1. Runs `cargo sqlx prepare`
2. Remember to perform migrations the the remote DB.


