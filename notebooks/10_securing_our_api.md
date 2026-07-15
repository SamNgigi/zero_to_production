# 10. Securing Our API

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

## 10.01. Authentication

##### 10.01.1.0. Skimming: What did you notice and why? Any Questions

**What?**  
Authentication Categories, that applications can use to authenticate their users
- Something they know (passwords)
- Something the have (Smartphone, authenticator apps, U2F Keys [Universal Second Factor-NFC, USB])
- Something they are (fingerprints, Face ID)


**Wny?**  
The breakdown is nice, clear, simplifying the general techniques used for authentication.  
Also a bit clear the distinction between
- Authentication - the who
- Authorization - the what

Need for Multi-factor authenticaion also made straightforward because of the flaws of the different authentication techniques when used  
in isolation.

**Questions?**   
None


### 10.01.0. Overview

##### 10.01.0.0.1. Deep Dive: Summarize, ELI5, Connect

_**Summary**_  
We have just added functionality to send out a newsletter to our subscribers. We need to restrict this functionality to only priviledged users  
that the application knows about.

How can an application identify priviledged users.
- Something they know e.g. a password
- Something they have e.g. an authenticator app on their smartphone
- Something they are e.g. Finger prints or Face ID

_**ELI 5**_  
Imagine going for a concert or getting into a club or confrence.  
There are roughly 4 groups of people
1. Organizers/Staff - Ticketing, Security, Waiters, Ushers etc
2. VIPs - Artists/DJs, Speakers
3. Attendees - People who are eligible to attend or enter the event
4. Eveyone Else - Who is either not eligible.

The category you belong to determines the level of access you have, perks & priviledges.

Keeping the concert analogy in mind, for our newsletter;
1. Only confirmed subscribers should receive the our newsletter. We sorted this out in the previous chapter.
2. At the bare minimum only app admins should be able to create and send a newsletter issues to confirmed subscribers.

The second one is the focus on this chapter. And for the next few sections we try to unpack the question
> _How do we best identify an admin for our newsletter?_

### 10.01.1. Drawbacks

_**Summary**_  
The guiding question;
> _How do we best identify an admin of our newsletter?_

Asked another way;
> _How do we best authenticate the identity of an admin?_

This is better wording because they an attacker can pretend to be an admin. So we must find a mechanism to acertain that an admin is  
truly and admin.  

Some of the options with their drawbacks include:

- Something they know e.g.  _Passwords_  
    - Must be sufficiently long - short passwords can be brute forced
    - Must be sufficiently private - i.e. only the admin knows the password and difficult for anyone else to guess based on what is publicly know about you.
    - Must be unique - a password unique to this service and not reused on another service. Because another service is compromised an attacker has universal access
      to other services that resuse the password
        
    Well this might be a challenge for users to remeber unique long, private passwords for 100s of online services.  
    This could be mitigated by password managers but the don't have the best ux.

<br/>

- Something they have e.g. _Authenticator Apps, Universal 2nd Factor Keys_
    - Losing a smartphone with the authenticator app or the U2F keys and an attacker can impersonate an admin.
<br/>

- Something they are e.g. _Fingerprints, Face ID_
    - Can't be "rotated", reset or changed.
    - Usually very sensitive data, that is often available to government agencies who might abuse it.



### 10.01.2. Multi-factor Authentication

_**Summary**_  
Turns out that the best way to authenticate the identity of a priviledged user is to use a combination of at least 2 of the above options to gain access.  
This is also know as **Multi-Factor Authentication**

## 10.02. Password-based Authentication (bulky)

### 10.02.1. Basic Authentication

##### 10.02.0.0. Skimming: What did you notice and why? Any Questions

_**What?**_  
[RFC2617](https://datatracker.ietf.org/doc/html/rfc2617#section-2) and [RFC7617](https://datatracker.ietf.org/doc/html/rfc7617)

Actix makes use of `HttpRequest`. Axum makes use of `Request` but the both use `HeaderMap`.
- Actix -> `actix_web::http::header::HeaderMap`
- Axum -> `http::HeaderMap`

`String` method `strip_prefix` and `splitn`



_**Why?**_  
- First time looking at an RFC
- Similar naming convention
- Cool string methods

_**Questions?**_  
- In the test `request_missing_authorization_are_rejected()` wondered why we aren't using `app.post_newsletters`. 
> Latter we find out that we update `app.post_newsletters` to pass authentication/authoriazation headers. This test needs to return  
> an error, if the appropriate headers are not included.

##### 10.02.0.0.1. Deep Dive: Summarize, ELI5, Connect

_**Summary**_  
We follow [RFC7617](https://datatracker.ietf.org/doc/html/rfc7617) scheme understand how to Password based authentication works.

_**ELI5**_  
MFA requires atleast 2 of the authentication factors.  
We will start with Password-based auth because it is simplest of the 3.

It basically boils down to 
1. Obtain a username and password
2. Verifying the username and password against existing stored credentials
3. Granting access.


In this section we are primarily focused on how to obtain the username and password and extract the necessary credentials.

So how do we do this?

Based on "The 'Basic' HTTP Authentication Scheme" from [RFC7617](https://datatracker.ietf.org/doc/html/rfc7617) the server (our API) reads the **Authorization Headers** from the incoming authentication request structured as follows
```curl
Authorization: Basic <base64 encoded credentails>
```
Say a client or a form in our application submits
> > username = "Aladdin",  
> > password = "open sesame",  
>
The client would need to construct the credential as
> > ```
> > "Aladdin:open sesame"
> > ```
Get a base64 encoding of the string so that the Authorization header in the request sent to our server/backend API is
> > ```
> > Authorization: Basic QWxhZGRpbjpvcGVuIHNlc2FtZQ==
> > ```
You can double check that the encoded string is truly _Aladdin:open sesame_ by decoding the base64 string [here](https://www.base64decode.org/)  
**Note** therefore that _**encoding does not mean encryption**_ because decoding the string reveals the password as clear text.

So the Basic Authentication scheme is based on the model that a client needs to authenticate itself using a _"user id"_ and _password_ for each
**protection space**/**realm** which is a specified as a string.

In our case the "realm" we are protecting is the `publish_newsletter` handler for the `/newsletters` endpoint that we can refer to as the _**publish 
realm**._ So upon receiveing a request to the protected realm with either non-existent or invalid credentials the server returns
```
HTTP/1.1 401 Unauthoriazed
Date: Fri, 19 June 2026 12:02:53 EST
WWW-Authenticate: Basic realm="publish"
```


#### 10.02.1.1 Extracting Credentials

##### 10.02.1.1.1. Deep Dive: Summarize, ELI5, Connect

_**Summary**_

- Write test `request_missing_authorization_are_rejected` to ensure unauthorized requests to `POST /newsletter` are rejected.
- Add `basic_authentication` to extract and return credentials for a request and call in `publish_newsletter`. This means we need a `Credential` struct to store retrieved credentials.
- Add `Auth` error variant to `PublishError` and map the appropriate status code `StatusCode::UNAUTHORIZED` and header value `WWW-AUTHENTICATE: Basic realm="publish""`
- Update test helper funcion `post_newsletters` to accept dummy credentials so that requests include authorization headers.

After the above we should have 
- a passing test that ensures that requests to `POST /newsletters` without "valid" credentials that we can verify are rejected with a 401.
- credentials that we can verify


_**Show me the code**_
```Rust
// tests/api/newsletter.rs
// [...]

#[tokio::test]
async fn requests_missing_authorization_are_rejected() {
    // Arrange
    let app = spawn_app().await;
    
    // Arrange

    
    // Arrange

    
}
```


This test should fail.

![image.png](10_securing_our_api_files/178eaafe-d54e-48db-83ba-00263824e253.png)

To make the test pass we'll need to extract auth credentials from the header and return appropriate response.  
Lets add a `basic_authentication` function to extract the credentials.  
We will use `actix_web::HttpRequest` to extract the authorization headers.
```Rust
//! src/routes/newsletter.rs
// [...]
use actix_web::HttpRequest;
use actix_web::http::{
    StatusCode, 
    header::{self, HeaderMap, HeaderValue}
};
use base64::Engine;
use secrecy::SecretString;

#[tracing::instrument(/**/)]
pub async fn publish_newsletter(
    // [...]
    request: HttpRequest,
) -> Result<HttpResponse, PublishError> {
    let _credentials = basic_authentication(request.headers())?;
}

struct Credential {
    username: String,
    password: SecretString,
}

fn basic_authentication(headers: &HeaderMap) -> Result<Credentials anyhow::Error> {
    let header_value = headers
        .get("Authorization")
        .context("The 'Authorization' header was missing.")?
        .to_str()
        .context("The 'Authorization' header was not a valid UTF8 string.")?;
    let base64encoded_segment = header_value
        .strip_prefix("Basic ")
        .context("The authorization scheme was not 'Basic'.")?;
    let decoded_bytes = base64::engine::general_purpose::STANDARD
        .decode(base64encoded_segment)
        .context("Failed to base64-decode 'Basic' credentials.")?;
    let decoded_credentials = String::from_utf8(decoded_bytes)
        .context("The decoded credential string is not valid UTF8.")?;

    let credentials = decoded_credentials.splitn(2, ":");
    let username = credentials
        .next()
        .ok_or_else(|| {
            anyhow::anyhow!("A username must be provided in 'Basic' auth.")
        })?
        .to_string();
    let password = credentials
        .next()
        .ok_or_else(|| {
            anyhow::anyhow!("A password must be provided in 'Basic' auth.")
        })?;
        
    Ok(Credentials{ username, SecretString::from(password)})
}
```

In the test we assert the the header contains `WWWW-Authenticate: Basic realm="publish"` when rejecting an unauthorized header.  
We add a `PublishError` variant `AuthError` that we'll match on returning a `StatusCode::UNAUTHORIZED` with the appropriate headers.
```Rust
//! src/routes/newsletter.rs

#[derive(thiserror::Error)]
pub enum PublishError {
    #[error("Authentication Failed")]
    Auth(#[source anyhow::Error]),
    #[error(transparent)]
    Unexpected(#[from anyhow::Error]),
}

impl ResponseError for PublishError {
    fn error_response(&self) -> HttpResponse {
        match self {
            PublishError::Unexpected => {
                HttpResponse::new(StatusCode::INTERNAL_SERVER_ERROR);
            }
            PublishError::Auth => {
                let mut response = HttpResponse::new(Status::UNAUTHORIZED);
                let header_value = HeaderValue::from_str(r#"Basic realm="publish""#)
                    .expect("Header value was not valid UTF8");
                response
                    .headers_mut()
                    .insert(header::WWW_AUTHENTICATE, header_value);
                response
            }
        }
    }

    // error_response calls status_code by default so now there's no need to implement it 
    // separately since we have a custom `error_response`
}

#[tracing::instrument(/**/)]
async fn publish_newsletter(/**/) -> Result<HttpResponse, PublishError> {
    let _credential = basic_authentication(request.headers()).map_err(PublishError::Auth)?;
    // [...]
}
```

Our `request_missing_authorization_are_rejected()` test should now be green. However our other tests that make use of `app.post_newsletters()` should now be failing.  

![image.png](10_securing_our_api_files/4a6a59b1-64fa-46fa-83a1-8c13c8f41e31.png)

We need to include the authoriaztion headers in the `app.post_newsletter` helper method.
```Rust
//! tests/api/helper.rs
// [...]

impl TestApp {
    async fn publish_newsletter(&self, body: serde_json::Value) -> reqwest::Response {
        let placeholder_cred  = Uuid::now_v7();
        reqwest::Client::new()
            .post(format!("{}/newsletters", &self.address))
            .basic_auth(placeholder_cred, Some(placeholder_cred))
            .json(&body)
            .send()
            .await
            .expect("Failed to execute publish newsletter post request in test.")
    }
    // [...]
}

```

### 10.02.2. Password Verification-Naive Approach

##### 10.02.2.0.0. Skimming: What did you notice and why? Any Questions

_**What?**_  
- `validate_credentials` we use `user_id` of type `Option<_>` mapping it to `Uuid` at the end.  
- How we are wiring up a test user with a password without having an explicit `User` type to work with.    
- Instatiating the `test_app` then calling `add_test_user` the finally returning `test_app`.  

_**Wny?**_  
Curious why we use `Option<_>` instead of `Option<Uuid>` directly instead?  
Insight into how best to setup a test user for authentication/authorization testing

_**Questions?**_  
Curious why we use `Option<_>` instead of `Option<Uuid>` directly instead?

##### 10.02.2.0.1. Deep Dive: Summarize, ELI5, Connect

_**Summary**_  
We create a `users` table & validate credentials were we retrieve from the DB a `user_id` of the extracted `username` and `password`  
combination.

We update our tests to create a new user and retrieve the user credentials that we then use to make authenticated request in `post_newsletter`.

_**Deep Dive**_  
We can break down this section into the following tasks.

1. Creating a users table that we'll use to query for a `user_id` given `username` and `password`
2. Add a `validate_credential` function that does the querying.
3. Adding an `add_test_user`  and `test_user` functions to our test helpers

Alright lets get to it.


**1. Create `users` table.**  
We run
```bash
sqlx migrate add create_users_table
```
We create an initial first draft of the `users` table.
```SQL
CREATE TABLE users (
    user_id UUID PRIMARY KEY,
    username TEXT NOT NULL UNIQUE,
    password TEXT NOT NULL
)
```

Make sure to run `sqlx migrate run` for the table to be create in our DB. This is to make  the static analysis of our code happy.  
In test this migration will be immediately run due to how we configure our db.  

**2. Add `validate_credentials`** method to `src/routes/newsletter.rs`
```Rust
//! src/routes/newsletter.rs
// [...]

pub async fn publish_newsletter(/**/) -> Result<HttpResponse, PublishError> {
    let credentials = basic_authentication(request.headers()).map_err(PublishError::Auth)?;
    let _user_id =  validate_credentials(credentials).await?;
}

// [...]

async fn validate_credentials(
    credentials: Credentials,
    db_pool: PgPool, 
) -> Result<Uuid, PublishError> {
    let user_id: Option<Uuid> = sqlx::query!(
        r#"
            SELECT user_id
                FROM users
            WHERE username = $1
                AND password = $2
        "#
        credentials.username,
        credentials.password.expose_secret()
    )
    .fetch_optional(&db_pool)
    .await
    .context("Failed to perform query to validate auth credentials")
    .map_err(PublishError::Unexpected)?;

    user_id
    .map(|row| row.user_id)
    .ok_or_else(|| anyhow::anyhow!("Invalid username or password"))
    .map_err(PublishError::Auth)
    
}
```

We add a tracing span to record who is calling `POST newsletters`
```Rust
//! src/routes/newsletter.rs
// [...]

#[tracing::instrument(
    name = "Publish newsletter"
    skip(db_pool, email_client, body, request),
    fields(username=tracing::field::Empty, user_id=tracing::field::Empty)
)]
pub async fn publish_newsletter(
    db_pool: web::Data<PgPool>,
    body: web::Json<BodyData>,
    email_client: web::Data<EmailClient>,
    request: HttpRequest,
){
    let credentails = basic_authentication(request.headers()).map_err(PublishError::Auth)?;
    tracing::Span::current().record(
        "username",
        tracing::field::display(&credential.username)
    );
    let user_id = validate_credentials(credentials).await?;
    tracing::Span::current().record(
        "user_id",
        tracing::field::display(&user_id)
    );
    // [...]
}
```

**3. Implement `add_test_user` test helper function.**  
We don't have a complete sign-up flow yet, for now we'll inject a test user directly to the database.
```Rust
//! tests/api/helpers.rs
// [..]

pub async fn spawn_app() -> TestApp {
    // [...]
    let test_app =  TestApp {/**/};
    add_test_user(&test_app.db_pool).await;
    test_app
}

async fn add_test_user(db_pool: &PgPool) {
    let test_user = Uuid::now_v7();
    sqlx::query!(
        r#"
            INSERT INTO users (user_id, username, password)
            VALUES ($1, $2, $3);
        "#,
        test_user ,
        test_user .to_string(),
        test_user .to_string()
    )
    .execute(db_pool)
    .await
    .expect("Failed to create test user.")
}
```

The we add a `test_user` helper method to `TestApp` to retreive the user we just created to replace the our `placeholder` in `post_newsletter` implementation.
```Rust
//! test/api/helpers.rs
// [...]

impl TestApp {

    async fn test_user(&self) -> (String, String) {
        let row  = sqlx::query!(
            r#"SELECT username, password FROM users LIMIT 1;"#,
        )
        .fetch_one(&self.db_pool)
        .await
        .expect("Failed to query test user.");

        (row.username, row.password)
    }

    async fn post_newsletters(&self, body: serde_json::Value) -> reqwest::Response {
        let (username, password) = self.test_user().await;
        reqwest::Client::new()
            .post()
            .basic_auth(username, Some(password))
            .json(&body)
            .send()
            .await
            .expect("Failed to perform newsletter post request in test.")
    }
}
```

### 10.02.3. Password Storage (bulky)

##### 10.02.3.0.1. Deep Dive: Summarize, ELI5, Connect

_**Summary**_  
This section is one of the more bulkier sub-sections of the current chapter. We can break it down into
1. Storing cryptographic hashes of a password instead of the raw passwords. We explore `SHA256` algorithm for effecient hashing of password strings
2. We unpack drawbacks and different flaws that attackers use to crack hashed passwords
3. We default to `argon2id` based password hashing + salting to store password string more securly and more computationally demanding to crack

This way we implement first the straightforward way we could store cryptographic passwords, then aftern analyzing the potential issues we  
implement the OWASP recommended way of cryptographically storing passwords.

#### 10.02.3.1. No Need To Store Raw Passwords

##### 10.02.3.1.0. Skimming: What did you notice and why? Any Questions

_**What?**_    
_Injective_
> Meaning for every unique input a function produces a unique output.  
> IF two outputs are the same, their original inputs must have been identical as well.
>
> **Example**  
> $f(x) = 2x + 1$ is an injective function because we get a unique output for every unique input  
> $f(x) = x^2$ is not an injective function because $x$ could be positive or negative and yeild the same output. e.g. 
> $-2$ or $2$

_**Wny?**_  
A password transformation password has to be injective such that if we store the tranformation on the database we can still verify against the  
actually password. i.e. $x \ne y$ then $f(x) \ne f(y)$ 

_**Questions?**_  
None.

##### 10.02.3.1.1. Deep Dive: Summarize, ELI5, Connect

_**Summary**_  
Right now we are validating the user credentials using the raw password stored in the database. If an attacker compromised the database, they would have  
access to all the passwords and can easily start impersonating users. They would also not be limited to our service, but other services where a user used the same  
password.

Because we are only checking for equality of passwords we can transform the raw password provided by a user and store that instead. Then use the transformed password  
for our equality check. The tranformation of the password has to meet the following criteria
1. The tranformation function on the raw password input has to be injective. This means for every unique input we have a unique output aka no two tranformation would produce the
   same output.
2. It should be impossible to do an inverse transformion. Once we produce an output it should be impossible to tell how the input can be derived, or how the output was produced.
   i.e. if an example transformation produced _"elloh"_ from _"hello"_ we can easily tell that our transform function reverses an input.
2. The input should completely uncorrelated to the output. In that a little change in the input, produces a completely different output that seems almost completely random  
   a.k.a. **avalanche effect**.

So what we need is a _**cryptographic hash function**_. 
- _cryptographic_ satisfies the uniformatiy property we discussed in the third criterion above. If the input string changes even a little the output is completely different and
  uncorrelated with the initial unchanged input string (_avalanche effect_)
- _hashing_ where we take a string from the input space and map it to a **fixed length** output. We differentiates hashing from encoding or encryption.
    - encoding means we can decode
    - encrypt means we can decrypt. 

    We cannot reverse a hash. Once one is generated we cannot get back to the original input string. This satisfies our second criteria.

What about _injectivity_?  
Hashing functions cannot be 100% injective. There is a high likelyhood though that if $f(x) == f(y)$ then $x==y$ even though not 100%. This is due to the possibility of collisions.  
Using a larger output space ($\ge$ 256 bits) makes collisions less likely. But again to completely down to 0%.

It is theoretically possible to create a perfect hashing algorith if the input length is fixed i.e. password length is capped

#### 10.02.3.2. Using A Cryptographic Hash

##### 10.02.3.2.0. Skimming: What did you notice and why? Any Questions

_**What?**_  
`format!("{:x}", password_hash)` to get hex-string.  
We add a `TestUser` to generate, store and access a test user that we use with `TestApp` in `spawn_app`


_**Why?**_  
That means to print something in hex we'd run `println!("{val:x}")`.   
Nice clean pattern to generate a test user with a cryptographically hashed password.

_**Questions?**_  
What about binary?  
As I suspected `println!("{val:b}")`

##### 10.02.3.2.1. Deep Dive: Summarize, ELI5, Connect

_**Summary**_  
Here we use SHA3-256 as our cryptographic hashing algorithm to hash our password for starters. We shall discuss the drawbacks for using it later.  
For now here's a great video on the Secure Hash Algorithm [SHA256](https://www.youtube.com/watch?v=orIgy2MjqrA&list=PLqppJXROAZOUr_3xmAk6LTSGdmE3dM3-c)

The main point is that we need to store password hashes instead of raw password and the sha3-256 algorithm provides a good starting point.

_**Deep Dive: ELI5**_

So how are we going to do this pratically?
1. We add the `sha3` crate
2. We add a migration to rename the `password` column to `password_hash` since we are no longer storing raw password text but a hash.
3. We update `validate_credentials` to query matching `password_hash` which means using `sha3` crate to generate a hash of our original password.
   As we do this we convert the `bytes` returned by `sha3` to a hex string that we can query with. Because `sha3` returns an array of bytes instead of a
   string/text.
4. We update our test helpers to store and retrieve a test user with a `password_hash` instead of a raw password.
   - Add a `TestUser` struct and implement a `generate` constructor and `store` method
   - Add a `test_user` field to `TestApp` and make the appropriate `generate` and `store` calls
   - Update `post_newsletter` to use `TestApp`'s `test_user`

Alright let's get to it.

**1. Add the `sha3` crate**
```bash
cargo add sha3
```

**2. Add rename password column migration**  
Because we want to store a `password_hash` instead of raw `password` text we rename the `users` table column to reflect this
```bash
sqlx migrate add rename_password_column
```

And the in the `.sql` script file
```SQL
ALTER TABLE users RENAME COLUMN password TO password_hash;
```

We can then run the migration with
```bash
sqlx migrate run
```

Our project stops compiling, with the error below

![image.png](10_securing_our_api_files/78bf449b-52a6-49b2-a0e8-52f0a27b2c01.png)

**3. Update `validate_credentials`**  
This should break our implementation because we were originally using `SELECT` on a `password` column.  
Also we the raw `credentials.password` wont work. We need to hash the extracted credentials to perform the query.  
Lets update accordingly.
```Rust
//! src/routes/newsletter.rs
// [...]

async fn validate_credentails(/**/) -> Result<Uuid, PublishError> {
   let password_hash = sha3::Sha3_256::digest(
       credentials.password.expose_secret().as_bytes()
   );
    
    // let password_hash = format!("{:x}", password_hash);
    let password_hash = hex::encode(password_hash);

    let user_id = sqlx::query!(
        r#"
            SELECT user_id
                FROM users
            WHERE username = $1 AND password_hash = $2;
        "#,
        credentials.username,
        password_hash,
    )
    .fetch_optional(db_pool)
    .await
    .context("Failed to perform query to validate auth credentials")
    .map_err(Publish::Unexpected)?;
    // [...]
}
```

A big difference between our implementation and the original is the use of the `hex` crate instead of `format!("{:x}", password_hash)` for getting a hex string from  
our password hash.

Without the conversion of the password_hash into a hex string we would get the error below, which in short tells us that a raw password hash returns a Array of `u8`s  
instead of `&str`.

![image.png](10_securing_our_api_files/c28528b9-1c25-49f6-ba71-adf602883571.png)

**4. Update test helper to use `password_hash` as opposed to raw `password`**  

We currently now have the errors below in a our tests

![image.png](10_securing_our_api_files/5fb236ab-c9bd-4cb3-aad6-5ca45d9a87e3.png)

This means we have to update how we are storing our initial `test_user`. We create a `TestUser` struct that we'll use to work  
with `test_user` credentials
<span id="sha3"></span>
```Rust
//! tests/api/newsletter.rs
// [...]

pub struct TestApp {
    // [...]
    pub test_user: TestUser
}

impl TestApp {
    pub async post_newsletter(/**/) -> reqwest::Response {
        reqwest::Client::new()
            .post()
            .basic_auth(&self.test_user.username, Some(&self.test_user.password_hash))
    }
}

pub async spawn_app() {
    // [...]
    let test_app = TestApp {
        // [...]
        test_user: TestUser::generate(),
    };
    test_app.test_user.store(&test_app.db_pool).await;
    test_app
}

struct TestUser {
    user_id: Uuid,
    username: String,
    password_hash: String,
}

impl TestUser {
    fn generate() -> TestUser {
        let user_id = Uuid::now_v7();
        TestUser {
            user_id,
            username: user_id.to_string(),
            password: user_id.to_string(),
        }
    }

    async fn store(&self, db_pool: PgPool) {
        let password_hash = sha3::Sha3_256::digest(
            self.password.as_bytes()
        );
        let password_hash = format!("{:x}" password_hash);
        sqlx::query!(
            r#"
                INSERT INTO users (user_id, username, password_hash)
                VALUES ($1, $2, $3);
            "#,
            self.user_id,
            self.username,
            password_hash
        )
        .execute(db_pool)
        .await
        .expect("Failed to perform insert test user query.");
    }
}
```

#### 10.02.3.3. Preimage Attack

##### 10.02.3.3.0. Skimming: What did you notice and why? Any Questions

_**What?**_  
Exponential time complexity ($2^n$) for a brute force attach to match an input string with SHA 256 hash of the password we are trying to hack.  
Where $n$ is the hash length in bits.  
Where $n > 128$ a preimage attack is unfeasible

_**Why?**_  
The name _Preimage attack_. 

_**Questions?**_  
None


##### 10.02.3.3.1. Deep Dive: Summarize, ELI5, Connect

_**Summary**_  
Does our current hashing mechanism using `sha3` make our password secure? 

We explore a couple of common attacks
1. Preimage attacks
2. Naive dictionary attacks
3. Dictionary attacks using rainbow tables for example.

Here we start with Preimage attacks.

What is a _preimage_ attack?  
Starting from the basic definition of a preimage, we are basically trying to reverse engineer the original password given a hash.  
We conclude that for a hash where its fixed length output is greatar than 128 bits it is commputationally unfeasible using brute force  
because the time complexity is $2^n$ where n in this case is $> 128$ which means $2^{(> 128)}$ 

In our case we are using SHA3-256 making our password has preimage resilient.

Great breakdown from ChatGPT differentiating between [preimage attacks from second preimage and collisions reslience](https://chatgpt.com/share/6a3b8d15-f6ac-83ea-9caa-f9f44fcbf3fc)

#### 10.02.3.4. Naive Dictionary Attack.

##### 10.02.3.4.0. Skimming: What did you notice and why? Any Questions

_**What?**_  
The math to estimate how long it would take to brute force an alphanumeric password shorter than 17 characters.  
Introduced to the concept of _rainbow tables_. Cool [video](https://www.youtube.com/watch?v=OzVzo9gtiec) and [wiki](https://en.wikipedia.org/wiki/Rainbow_table)

_**Why?**_  
Heard about rainbow tables alot when it comes to securing passwords

_**Questions?**_  
None


##### 10.02.3.4.1. Deep Dive: Summarize, ELI5, Connect

_**Summary**_  
What about when we reduce the search space by making some assumptions about the input password, such as length and exclude symbols?  
Here I think the book means naive brute force dictionary attack where we use a 17 character alphanumeric password and walks through the math.  
The summary is though that if the input space is reduced enough advance GPU can allow brute force speed ups where we are able to generate billions  
of hashes for different alphanumeric character combinations.

The book gives an example of how researchers managed to compute ~900 million SHA-512 hashes per second using GPU.

However a brute force attack is easy to frustrate by including something as simple as capitalized alphabets in the alphanumeric set.

Really great [video](https://www.youtube.com/watch?v=7U-RbOKanYs&t=715s) on password cracking via native dictionary brute force and sophisticated dictionary attacks

#### 10.02.3.5. Dictionary Attack

##### 10.02.3.5.0. Skimming: What did you notice and why? Any Questions

_**What?**_  
Cryptographic algorithms we discussed until now are designed to be fast. Meaning it is possible to conpute the inverse transform on consumer hardware

_**Why?**_  
Having a cryptographic algorithm that computes hashes fast/effectively is actually a flaw. We need to make the hashes hard/slow to compute inorder to make it
even more difficult for attackers who may have access to dictionaries of hashes who may use rainbow tables.

_**Questions?**_  
None

##### 10.02.3.5.1. Deep Dive: Summarize, ELI5, Connect

_**Summary**_  
A non-naive dictionary attack takes advantage of widely available password and hash combinations dictionaries of known password leaks. Here an attacker  
takes advantage of these dictionaries + compute to be able to crack password hashes to get original password.

The key insight here is that SHA3 hash functions are computationally efficient allowing us compute hashes very fast and this is only made better with dictionaries.  

We need something much **slower** that makes it computationally expensive to generate hashes. Such that even when someone has the hashes it is computationally unfeasible  
to hash a dictionary of password that generate the hashes the were compromised.

#### 10.02.3.6. Argon2

##### 10.02.3.6.0. Skimming: What did you notice and why? Any Questions

_**What?**_  
- Open Web Application Security Project ([OWASP](https://cheatsheetseries.owasp.org/index.html))
- `password_hash` crate

_**Why?**_  
Should be good to revisit and explore the material. We start with the [Password Storage Cheat Sheet](https://cheatsheetseries.owasp.org/cheatsheets/Password_Storage_Cheat_Sheet.html)  when looking at  
Argon2. Other cheatsheets also looking quite information rich.

Seems  like we replace what we were doing before with `sha3::SHA_256::digest` with 2 crates, `argon2` and `password_hash`, and we need to include a salt inorder to  
appropriately hash out password.

_**Questions?**_  

None

##### 10.02.3.6.1. Deep Dive: Summarize, ELI5, Connect

_**Summary**_  

We take a look at OWASP's summary recommendation when it comes to password storage

<div style="background-color: #313B51; color: white; padding: 15px; border-radius: 15px;">

- Use [Argo2id](https://cheatsheetseries.owasp.org/cheatsheets/Password_Storage_Cheat_Sheet.html#argon2id) with a minimum configuration of 19MiB of memory
  an iteration count of 2, and 1 degree of parallelism.
- If [Argon2id](https://cheatsheetseries.owasp.org/cheatsheets/Password_Storage_Cheat_Sheet.html#argon2id) is not available use
  [scrypt](https://cheatsheetseries.owasp.org/cheatsheets/Password_Storage_Cheat_Sheet.html#scrypt) with a minimum CPU/memory cost parameter of $2^17$,
  a minimum block size of 8(1024 bytes), and a parallelization of 1.
- For legacy system using [bcrypt](https://cheatsheetseries.owasp.org/cheatsheets/Password_Storage_Cheat_Sheet.html#bcrypt), use a work factor of 10 or
  more and with a password limit of 72 bytes
- If FIPS-140 compliance is required, use [PBKDF2](https://cheatsheetseries.owasp.org/cheatsheets/Password_Storage_Cheat_Sheet.html#pbkdf2) with a work
  factor of 600,000 or more and set with an internal hash function of HMAC-SHA-256.
- Consider using a [pepper](https://cheatsheetseries.owasp.org/cheatsheets/Password_Storage_Cheat_Sheet.html#peppering) to provide additional defense
  in depth(though alone, it provides no additional secure characteristics) 
  
</div>

This is an updated excerpt from the [OWASP Password Storage Cheat Sheet](https://cheatsheetseries.owasp.org/cheatsheets/Password_Storage_Cheat_Sheet.html#argon2id).
Make sure to check the differences from the original excerpt added in the Zero To Production book.

Really good summative [video](https://www.youtube.com/watch?v=qQAhprPM5lw) resource on why argon2.

We then go about adding Argon2id to replace the password hashing we did with `sha3`. To do this we;
1. Add the `argon2` crate and instantiate our argon `hasher` with the requisite configs.
2. Add a `salt` column to our `users` table to make our password hashes even more robust.
3. Update our sqlx query to return `user_id`, `expected_password_hash` and `salt` so that we can hash our extracted `credential.password` together with the store `salt`
   before doing an equality check with our `expected_password_hash`
4. We then remove our initial hasher and prefer to use argon's `PasswordHash::new` to return a **PHC** string formated `expected_password_hash` that we use to verify
   our passed in credentials with `Argon2::default().verify_password`
5. We than add a migration to `drop_salt_column_from_users`.
6. Finally we update our `TestUser`'s `store` method to store an argon2id `password_hash`

Notice that we add a `salt` column to illustrate its usefulnes and then remove it because argon handles it for us.  
Also notice the the above tasks cover the next few sections.

Alright. lets get into it

**1. Add the `argon2` dependency and initialize our argon `hasher` with requisite configs**.  
```bash
cargo add argon2 --feature std
```

Then in we update `src/routes/newsletter.rs` as follows, following the OWASP guidelines on the recommended configuration.
```Rust
// src/routes/newsletter.rs
// [...]
use argon2::{Algorithm, Argon2, Version, Params};

// [...]

async fn validate_credentials(/**/) -> Result<Uuid, PublishError> {
    let hasher = Argon2::new(
        Algorithm::Argon2id,
        Version::0x13,
        Params::new(19_000, 2, 1, None)
            .context("Failed to build Argon2 params")
            .map_err(PublishErr::Unexpected)?
    );

    // [...]
}
```

`Argon2` implements a `PasswordHasher` trait (a re-export from the `password-hash` crate) that requires an additional `salt` parameter.
```Rust
//! password_hash/traits.rs

pub trait PasswordHaser {
    // [...]
    fn hash_password<'a, S>(
        &self,
        passwrod: &[u8],
        salt: &'a S
    ) -> Result<PasswordHash<'a>>
    where 
        S: AsRef<str> + ?Sized;
}
```

#### 10.02.3.7. Salting

##### 10.02.3.7.0. Skimming: What did you notice and why? Any Questions

_**What?**_
We re-order `validate_credentials`, querying the `user_id`, `password_hash` (_expected_password_) and `salt` first before hashing the password  
got from the request to do a comparison against.

_**Why?**_
Was wondering where we insert a hashed & salted password, and realized that in the handler we are only evaluating the credentials that come with the  
`publish_newsletter` requests are valid.

_**Question?**_  
None

##### 10.02.3.7.1. Deep Dive: Summarize, ELI5, Connect.

_**Summary**_  
_What is salting and why do we need it?_  
Imagine a compromised user database where an attacker is able to access user password hashes. Our sophisticated attacker uses dictionaries and rainbow tables and  
is able to get a number of raw password strings from the hashes. What can we do to safe gaurd users if there passwords were compromised?

This is where salting helps us.  
A salt is a random string that we can append to a user password that allows us to generate a completely new hash. Each user gets their own salt, ensuring that even if
2 users had identical password, once we append our salt, there resulting hashes will be completely different.

If our application was compromised, all we have to do is generate new salts, while maintaining the same passwords. If the user used the same password on other applications  
and they also apply a salt, comprised password hashes are no longer of any usefulness to an attacker.

Adding a salt makes it computationally unfeasible to rehash compromised passwords.

We'll start by adding a `salt` column to our `users` table.  
What if an attacker is able to get a hold of the password_hashes + salts?
Well, they have to compute $dictionary\_size \times n\_user$ hashes instead of just the $dictionary\_size$. Furthermore pre-computing the hashes is no longer an option  
<span>&mdash;</span> this buys us time to detect the breack and take action (e.g. force a password reset for all users)

**2. Add a `salt` column to our `users` table to make our password hashes even more robust.**  

Let's add a `salt` column to our users table.
```bash
sqlx migrate add add_salt_to_users
```
```SQL
ALTER TABLE users ADD COLUMN salt TEXT NOT NULL UNIQUE;
```

Then we update `validate_credentials` query by retreving `user_id`, `password_hash` and `salt`.  
We'll need the `salt` when hashing the extracted credentials password from the request. Then we'll do an equality check with the queried password.
```Rust
//! src/routes/newsletter.rs
// [...]
use argon2::{
    Algorithm, Argon2id, PasswordHasher, Params, Version,
    password_hasher::SaltString
};

//[...]

async fn validate_credentials(/**/) -> Result<Uuid, PublishError> {
    let row: Option<_> = sqlx::query!(
        r#"
            SELECT user_id, password_hash, salt
                FROM users
            WHERE username = $1
        "#,
        credentials.username,
    )
    .fetch_optional(&db_pool)
    .await
    .context("Failed to perform a query to retrieve stored credentials.")
    .map_err(PublishError::Unexpectd)?;

    let (user_id, expected_password_hash, salt) = match row {
        Some(row) => (row.user_id, row.expected_password_hash, row.salt)
        None => return Err(PublishError::Auth(anyhow::anyhow!("Invalid Username")));
    }

    let hasher = Argon2::new(
        Algorithm::Argon2id,
        Verson::0x13,
        Params(19_000, 2, 1, None)
            .context("Failed to build argon2 params.")
            .map_err(PublishError::Unexpected)?
    );

    // NEW: We need to build a string salt from 
    let salt = SaltString::encode_b64(salt.as_bytes())
        .context("Failed to base-64 encode salt")
        .map_err(PublishError::Unexpected)?;

    let password_hash = hasher.hash_password(
        credentials.password.expose_secret().as_bytes(),
        &salt
    )
    .context("Failed to hash password.")
    .map_err(PublishError::Unexpected)?;

    let password_hash = hex::encode(password_hash.hash.unwrap());

    if expected_password_hash != password_hash {
        Err(PublishError::Auth(anyhow::anyhow!("Invalid password")))
    } else {
        Ok(user_id)
    }
}
```

**Note:** That we've had to encode the retreived `salt` using `SaltString::encode_b64` to return a `SaltString` that we can use when calling `.hash_password`. The original implementation  
did not require this.

In our case our code compiles and we do not get a `the trait LowerHex is not implemented error.` This is because we are using `hex::encode` on the resulting `password_hash.hash.unwrap()`. 

However we will still be missing out on the advantages that are offered by using a PHC String Format.


#### 10.02.3.8. PHC String Format

##### 10.02.3.8.0. Skimming: What did you notice and why? Any Questions

_**What?**_
- **PHC** _string_ - Password Hashing Competition String.
- We remove the _salt_ column that we added in the previous section.

_**Why?**_
- Interesting to know the PHC string format is the result of winning a competition on secure password hashing and storage.
- Argon2 takes care of adding a random salt, via the `PasswordHash::new` that returns to us a valid PHC string format password hash.

_**Question?**_  
None


##### 10.02.3.8.1. Deep Dive: Summary, ELI5, Connect.

_**Summary**_  

_What is the PHC String format and why do we need/use it?_  
PHC stands for $\textsf{Password Hasing Competition}$, and the string format allows us to encode, the hashing algorithm used to generate the hash, the version &  
core params, the salt and the password_hash itself.

Why do we need it?   
Newer and better hashing algorithms can arise, are existing algorithms upgraded to higher versions, or params might need tweaking as hardware improvments happen.
> For example in the book we originally start with a `m_cost` of 15MiB. As of today as per the OWASP cheatsheet the `m_cost` is now 19MiB.

To account for the updates, and changes that may occur in password hashing we need a way to maintain stable backward compatibility while still making upgrades simple.
Take a scenario where older users of our platform use a different hashing algorighm version compared to new users when an application upgrades.  
PHC provides a reliable way to maintain hashed passwords while allowing us flexibility to upgrade, because a password hash's meta data is enconded in the string itself,
so any authentication routine will know how to hash and verify passwords, allowing for seemless migration.

An example of a PHC string looks like this:
```bash
#${algorithm}${algorithm version}${comma separated algorithm params}${hash}${salt}
$argon2id$v=19$m=65536,t=2,p=1$gZiV/M1gPc22ElAH/Jh1Hw$CWOrkoo7oJBQ/iyh7uJ0LO2aLEfrHwTWllSAxT0zRno
```

Alright onto the how.
`argon2` crate exposes `PasswordHash` which is Rust's implementation of the PHC string format. For verification we can rely on `PasswordVerifier` trait exposed by `argon2` as  
well. By passing the expected password via `PasswordHash`, `Argon2` can infer the what load paramereters and salt were used to verify if the password candidate is a match.

We update `validate_credentials` as follows;
```Rust
//! src/routes/newsletter.rs
// [...]
use argon2::{Argon2, PasswordHash, PasswordVerifier};

async fn validate_credentials(/**/) -> Result<Uuid, PublishError> {
    let row: Option<_> = sqlx::query!(
        r#"
            SELECT user_id, password_hash
                FROM users
            WHERE username = $1
        "#,
        credentials.username,
    )
    .fetch_optional(db_pool)
    .await
    .context("Failed to perfom query to retrieve stored credentials.")
    .map_err(PublishError::Unexpected)?;

    let (user_id, expected_password) = match row {
        Some(row) => (row.user_id, row.expected_password),
        None => return Err(PublishError::Auth(anyhow::anyhow!("Invalid Username.")))
    };
    
    let expected_password_phc_format = PasswordHash::new(&expected_password)
        .context("Failed to parse hash in PHC String format.")
        .map_err(PublishError::Unexpected)?;
    
    Argon2::default()
        .verify_password(
            credentials.password.expose_secret().as_bytes(),
            &expected_password_phc_format
        )
        .context("Invalid Password.")
        .map_err(PublishError::Auth)?;

    Ok(user_id)
}
```

This means that we don't need to store and query the `salt` separately to create the hash since it is already part of the PHC string format. As we'll see later, we'll add the  
`salt` before generating the `password_hash` for storage. This means that the `salt` will already be apart of the `password_hash` due to storing it in the PHC string format. 
We can go ahead and drop the `salt` column.
```bash
sqlx migrate add drop_salt_from_users
```
```
ALTER TABLE users DROP COLUMN salt;
```

Our project should compile successfully.  
What about out tests?
Two of them are failing.

![image.png](10_securing_our_api_files/42e83dc7-0345-4ae4-b1c8-ebcec2c97502.png)

We run `RUST_LOGS=true cargo test --test  -- newsletters_are_not_delivered` to see the logs we see;

![image.png](10_securing_our_api_files/1f87830e-81cc-4415-9efc-78a11740aefd.png)

When we look at the password generation code of our `test_user` [here](#sha3) we see that we are still storing a `sha3` hash instead of `argon2` PHC string formatted hashes.  
Lets fix this.

```Rust
//! test/api/newsletter.rs
// [...]
use argon2::{Argon2, PasswordHasher, password_hash::{SaltString, rand_core::OsRng}};

// [...]

impl  TestUser {
    // [...]
    async fn store(&self, db_pool: PgPool) {
        let salt = SaltString::generate(&mut OsRng);
        let password_hash = Argon2::default()
            .hash_password(
                self.password.expose_secret().as_bytes()
                &salt
            )
            .expect("Failed to hash password")
            .to_string();

        sqlx::query!(
            r#"
                INSERT INTO users (user_id, username, password_hash)
                VALUES ($1, $2, $3)
            "#,
            self.user_id,
            self.username,
            password_hash
        )
        .execute(db_pool)
        .await
        .expect("Failed to perform query to create a test user");
    }
}
```
All the test suites should passnow.

### 10.02.4. Do Not Block The Async Executor

#### 10.02.4.1. Overview

##### 10.02.4.0.0 Skimming: What did you notice and why? Any Questions

_**What?**_  
- Blocking tasks, cooperative scheduling and async tasks.
- Lifetime error we get because of `Password::new(&expected_password_hash)`

_**Wny?**_  
- Seems like blocking tasks are asynchronous tasks that take too long to return a result and therefore are meant to yeild back control to the
  scheduler for other async tasks to make progress without being blocked. Tasks that could block are meant to be run in a separate threadpool
  `tokio::task::spawn_blocking` so that they don't interfere with other async tasks.

- The lifetime issue is somewhat unclear. Would be good to revisit.

_**Questions?**_  
None


##### 10.02.4.0.1. Deep Dive: Summarize, ELI5, Connect

_**Summary**_  
We primarily unpack how async works and how to handle blocking tasks like CPU intensive tasks.  
Here's a [link](https://claude.ai/share/44ed5331-1cb7-4acd-905d-cf1f4f4a5681) to a Claude conversation deep dive into async vs blocking tasks, threads & processes.

1. We first inspect the how long password verification takes.
2. Then understand the nature of blocking tasks
3. Add `spawn_blocking` block and call `Argon2.verify_password`
4. Extract out returning PHC string parsing and password verification into its own function.
5. Add `span_blocking_with_tracing` helper function to `telementry.rs` module, and use it on `validate_credentials`



**1. Inspecting how long password verification takes**

To do this we will do a bit of refactoring. 
- Add `.with_span_events(FmtSpan::Close)` in `src/telemetry.rs`
- We'll separate the query to its own function `get_stored_credentials`. W
- Compute time elapsed for password verification.

_Add _`with_span_events(FmtSpan::CLOSE`) to `fmt::layer` to`get_subscriber` in `src/telemery.rs`._
```Rust
//! src/telemetry.rs
// [...]

pub fn get_subscriber<Sink>(/**/) -> impl Subscriber + Send + Sync 
where 
    Sink: for <'a> MakeWriter<'a> + Send + Sync + 'static
{
    // [...]
    let formatting_layer = fmt::layer()
        .json()
        .with_writer(sink)
        .with_span_events(FmtSpan::CLOSE)
        .with_current_span(true);

    // [...]
}
```

_Compute time elapsed for password verification_ +  

```Rust
//! src/routes/newsletter.rs
// [...]

#[tracing_instrument(name="Validate Credentials", skip(db_pool, credentaisl))]
async fn validate_credentails(db_pool: &PgPool, credentials: Credentials) -> Result<Uuid, PublishError> {
    let (user_id, expected_password_hash) = get_stored_credentials(db_pool, username)
        .await
        .map_err(PublishError::Unexpected)?
        .ok_or_else(|| PublishError::Auth(anyhow::anyhow!("Invalid Username".)))?;
    
    let start_time = std::time::Instant::now();
    let outcome = Argon2::default().verify_password(
        credentials.password_hash.expose_secret().as_bytes(),
        &expected_password_hash
    );
    tracing::info!(elapsed_milliseconds = start_time.elapsed().milliseconds, "Verifed password hash.");
    outcome
    .context("Invalid Password")
    .map_err(PublishError::Auth)?;

    Ok(user_id)
}

#[tracing::instrument(name="Get Stored Credentials", skip=(db_pool, username))]
async fn get_stored_credentials(db_pool: &PgPool, username: &str) -> Result<Option<(user_id, SecretString)> anyhow::Error>{
    let row = sqlx::query!(
        r#"
            SELECT user_id, password_hash
                FROM users
            WHERE username = $1;
        "#
        username
    )
    .fetch_optonal(db_pool)
    .await
    .context("Failed to perform query to retreive stored credentials.")?
    .map(|r| (r.user_id, r.password_hash));

    Ok(row)
} 
```

We can inspect the logs by running the command;  
``` bash
TEST_LOG=true cargo test --quiet --release newsletters_are_delivered | grep "Verified password hash" | jq -R "fromjson"
```
![image.png](10_securing_our_api_files/70339806-d7be-4314-9ae3-780036f0f3fe.png)

We can see that it took about 27 milliseconds. This is very likely to cause issues under load causing the infamous **blocking problem**.  
> **Note**: In the book the elapsed time is ~ 10ms. Our is significantly higher at almost 3 times the time elapsed.  
> Maybe we can attribute this to the `19MiB` memory cost for our default compared to the original `15MiB` memory cost for Argon2.

###### Learning-1: Making Spans Emit A Close Event

By adding `.with_span_event(FmtSpan::Close)` in the `fmt` in the `telemetery.rs` module, we make spans emit a close event systematically whenever we build the JSON fmt layer in `get_subscriber`.  
This allows us to configure how synthesized events are emitted at points in a `spans` lifecycle. `FmtSpan::CLOSE` events will be synthesized when a span closes, and the event field will contain  
`busy time` (total time for which a span was entered) and `idle time` (time for which the span existed but was not entered)
Might proceed with this approach because of the next section on _User Enumeration_ when tracking the time it takes for a non existent user login vs existing user login.

More on that [here](https://claude.ai/share/05b6d6c9-d3be-4df7-89fb-7865fbd6a5e5) from a conversation with Claude.

###### Learning-2: Formating tracing output in pretty json

Below is an example where we want to a json log output with the key word _"Verified"_ for a speciific test that has the substring   
`newsletters_are_delivered`
```bash
TEST_LOG=true cargo test --quiet --release newsletters_are_delivered 2>&1 | grep "Verified" | jq -R 'fromjson?'
```

This produces the following output
```bash
➜ TEST_LOG=true cargo test --quiet --release newsletters_are_delivered 2>&1 | grep "Verified" | jq -R 'fromjson?'
{
  "timestamp": "2026-06-26T11:00:46.398758Z",
  "level": "INFO",
  "fields": {
    "message": "Verified password hash",
    "elapsed_ms": "27"
  },
  "target": "zero2prod::routes::newsletters",
  "span": {
    "name": "Validate credentials"
  },
  "spans": [
    {
      "http.client_ip": "127.0.0.1",
      "http.flavor": "1.1",
      "http.host": "127.0.0.1:51900",
      "http.method": "POST",
      "http.route": "/{name}",
      "http.scheme": "http",
      "http.target": "/newsletters",
      "http.user_agent": "",
      "otel.kind": "server",
      "otel.name": "POST /{name}",
      "request_id": "699bc4fd-f2b0-4ef7-990a-bec001ae0fd1",
      "name": "HTTP request"
    },
    {
      "username": "019f0396-dc48-7451-b562-c3d60c4669a1",
      "name": "Publish newsletter"
    },
    {
      "name": "Validate credentials"
    }
  ]
}
```

**Note:** The order of the `grep` command matters. 
Running the command below;
```bash
TEST_LOG=true cargo test --quiet --release newsletters_are_delivered 2>&1 | jq -R 'fromjson?' | grep "Verified" 
```
results in;
```bash
➜ TEST_LOG=true cargo test --quiet --release newsletters_are_delivered 2>&1 | jq -R 'fromjson?' | grep "Verified"
    "message": "Verified password hash",
```

Conversation with [Claude](https://claude.ai/share/8b6bf501-9a1d-4cad-b524-fbab4499b617) where I learn about adding the `jq .` to add json formating to tracing outputs.

##### Blocking problem and Cooperative Scheduling

Rough shape of this section.

Rust `async`/`await` is built around a concept known as **cooperative scheduling**.
Let imagine a simple example `my_fn` 
```Rust
async fn my_fn() {
    a().await;
    b().await;
    c().await;
}
```

`my_fn` returns a `Future`. When a `Future` is awaited `tokio` our async runtime enters into the picture and begins to poll the returned `Future`.  
Assuming that the inner function within `my_fn` don't have nested await calls within themselves, polling `my_fn`'s `Future` can be represented by 
tracking the state of the `Future` represented by the enum below.  
```Rust
enum MyFnFuture {
    Initialized,
    CallingA,
    CallingB,
    CallingC,
    Complete,
}
```
We have different states in `MyFnFuture` for each `.await` in our `async` function body.  
Each `.await` call returns either a `Ready` or `Pending` state and then **yields** control back to the executor. That is why `.await` calls are often named **yield points**.

The executor can then choose to poll the same future again and progress to the next `.await` if a task is complete, or to prioritize making progress on another tasks.   
This is how async runtimes like `tokio` make progress **concurrently** on multiple tasks $\textemdash$ by continously parking and resuming each of them.

The underlying assumption is that most _async_ tasks are performing some kind I/O work $\textemdash$ most of their execution time is spent waiting on soemthing else to happen (.e.g  
the OS notifying us that a file is ready to be read on a socket.) Therefore we can _effectively_ perform many more tasks concurrently than we would achieve by a dedicated parallel  
unit of execution.

This model works great assuming that tasks **cooperate** and yeild back control to the executor quickly and frequently ideally less than 10 - 100 microseconds ($\micro$s). If the async tasks  
take longer or even worse never yeild back control, the executor cannot make progress on other tasks. This is what we call the blocking problem where a task is blocking the executor/  
async thread.

CPU heavy workloads are the common culprit for blocking tasks, because they do not yeild back control to the executor until they are complete, that are likely to take longer than 1ms to complete.  
A good example is our password verification with `argon` which we specifically choose because it is **computationally demanding**.

To play nicely with tokio we offload CPU intensive work to a separate dedicated thread pool for blocking tasks using `tokio::task::spawn_blocking`. This ensures that we do not interfere with  
scheduled async tasks.

This table is just FYI.


| Prefix | Symbol | Fraction of a Second | Scientific Notation | Numeric Value |
| :--- | :--- | :--- | :--- | :--- |
| **Millisecond** | ms | One thousandth | 10⁻³ s | 0.001 seconds |
| **Microsecond** | μs | One millionth | 10⁻⁶ s | 0.000001 seconds |
| **Nanosecond** | ns | One billionth | 10⁻⁹ s | 0.000000001 seconds |
| **Picosecond** | ps | One trillionth | 10⁻¹² s | 0.000000000001 seconds |


Alright. Lets offload password verification to an blocking threadpool.
```Rust
//! src/routes/newsletter.rs
// [...]

#[tracing::instrument(/**/)]
async fn validate_credentials(/**/) ->  Result<Uuid, PublishError> {
    // [...]
    tokio::task::spawn_blocking(move || {
        Argon2::default()
            .verify_password(
                credentials.password_hash.expose_secret().as_bytes(),
                &expected_password_phc_format
            )
    })
    .await
    .context("Failed to spawn blocking task thread")
    .map_err(PublishError::Unexpected)?
    .context("Invalid Password")
    .map_err(PublishError::Auth)?;
}
```
Unfortunately this does not compile leading to the error below.

![image.png](10_securing_our_api_files/3685eb77-f985-4037-a561-39e8e03e70fc.png)

The error informs us that `expected_password` does not live long enough when it is used by `PasswordHash` to generate our PHC string formated hash.  
`PasswordHash` holds a reference to the string it was parsed from, meaning that we would have to guarantee that the reference lives long enough when we used the resultant  
formatted hash in our closure in the blockin task.

To work around this lets wrap both the PHC string format parsing and hash verfication in a separate function that we can then call in the `spawn_blocking`.
```Rust
//! src/routes/newsletter.s
// [...]

// [...]

#[tracing::instrument(/**/)]
async fn validate_credentials(/**/) -> Result<Uuid, PublishError> {
    // [...]

    tokio::task::spawn_blockin(move || {
        verify_password_hash(expected_password, credentials.password)
    })
    .context("Failed to spawn blocking task thread.")
    .map_err(PublishError::Unexpected)??;

    // [...]
}

// [...]

#[tracing::instrument(name="Verify Password Hash", skip(expected_password, password_candidate))]
async fn verify_password_hash(
    expected_password: SecretString,
    password_candidate: SecretString,
) -> Result<(), PublishError> {
    let expected_password_phc_format = PasswordHash::new(expected_password.expose_secret())
        .context("Failed to parse hash in PHC string format.")
        .map_err(PublishError::Unexpected)?;

    Argon2::default()
        .verify_password(
            password_candidate.expose_secret().as_bytes(),
            &expected_password_phc_format,
        )
        .context("Invalid Password.")
        .map_err(PublishError::Auth)
}
```

This should compile.

#### 10.02.4.1. Tracing Context Is Thread-Local

##### 10.02.4.1.0 Skimming: What did you notice and why? Any Questions

_**What?**_
- Attaching current span to newly spanwed thread.

_**Why?**_
- We write a custom function in order to do this utilizing `JoinHandle`, `FnOnce`, and trait bound definitions. A nice pattern to learn from.

_**Questions?**_  
None



##### 10.02.4.1.1. Deep Dive: Summarize, ELI5, Connect

_**Summary**_  

Because `verify_password_hash` is being called in a separate thread (via `tokio::task::spawn_blocking`), it does not inherit the tracing properties from its parent span corresponding to the request, e.g `request_id`, `http.method`, `http.target` e.t.c. as shown below.

![image.png](10_securing_our_api_files/5407cc66-078b-4175-bcc1-09e60a212c78.png)

Looking at `tracing`'s docs, we know
<div style="background-color: #313B51; color: white; padding: 15px; border-radius: 15px; border: 1px solid white;">

<a href ="https://docs.rs/tracing/latest/tracing/span/index.html#span-relationships">Spans</a> form a tree structure $\textemdash$ unless it is a  
root span, all spans have a _parent_, and may have one or more _children_. When a new is span is created, the **current span** becomes the new  
span's parent.

</div>
</br>

The current span is the one returned by `tracing::Span::current` $\textemdash$ let's check its [documentation](https://docs.rs/tracing/latest/tracing/struct.Span.html#method.current)

<div style="background-color: #313B51; color: white; padding: 15px; border-radius: 15px; border: 1px solid white;">

Returns a handle ot the span <a href="https://docs.rs/tracing/latest/tracing/trait.Subscriber.html#method.current_span">considered by the `Subscriber`</a> to be the current span.

If the _subscriber_ indicates that it does not track the current span, or that **the thread from which this function is called is not currently  
inside a span**, the returned span will be disabled.

</div>

**current span** actually means "active span for the current thread", and this is the reason why we aren't inheriting any properties from parent thread because we are spawning a new thread dedicated to running blocking task and then calling `verify_password_hash`.

By explicitly attaching the current span to the newly spawned thread, we work around the issue

```Rust
//! src/routes/newsletter.rs
// [...]

// [...]

#[tracing::instrument(/**/)]
async fn validate_credentials(/**/) -> Result<Uuid, PublishError> {
    // [...]
    // This executes before spawning the new thread
    let current_span = tracing::Span::current();
    tokio::task::spawn_blocking(move || {
        // We then pass ownership to it into the closure and explicitly executes all our computation
        // within its scope
        current_span::in_scope(|| verify_password_hash(expected_password, credentials.password));
    })
    .context("Invalid Password")
    .map_err(PublishError::Auth)?;
    // [...]
}
```

![image.png](10_securing_our_api_files/c74d7096-9655-4ddc-9eb4-715f94bb6e6c.png)

Because we might want to reuse this functionality $\textemdash$ attaching current span to a blocking task thread, let's add a helper function to  
`src/telementry.rs` for this.
```Rust
//! src/telementry
// [...]
use tokio::task::JoinHandler;

fn spawn_blocking_with_tracing<F, R>(f: F) -> JoinHandler<R>
where
    F: FnOnce() -> R + Send + 'static,
    R: Send + 'statix
{
    let current_span = tracing::Span::current();
    tokio::task::spawn_blocking(move || current_span.in_scope(f)) 
}
```

We then call it the helper function in `src/routes/newsletter.rs` as follows.
```Rust
// src/routes/newsletter.rs
// [...]
use crate::telemetry::spawn_blocking_with_tracing;

#[tracing::instrument(/**/)]
async fn validate_credentials(/**/) -> Result<Uuid, PublishError> {
    // [...]
    spawn_blocking_with_tracing(move || 
        verify_password_hash(expected_password, credentials.password)
    )
    .await
    .context("Invalid Password")
    .map_err(PublishError::Auth)??;
}
```

We can now easily reach for the helper every time we need to offload some CPU-intensive computation to a dedicated threadpool.

### 10.02.5. User Enumeration

##### 10.02.5.0. Skimming: What did you notice and why? Any Questions

_**What?**_
- Timing attacks.
- Realizing that Invalid username and/or password is a good generic way of rejecting both non-existent accounts or password-username mismatch.
  Addressing timing attacks makes it difficult to differentiate between invalid credentials and non-existent credentials.

_**Why?**_
- A clever tactic to identify valid users of an application vs non-existent in Saas applications with specific registered domains as login credentials ( e.g. user@somesaas.com)
-  Imagining a scenario where we know a valid username/email because of timing attacks and initiate a password reset to gain control of the account.

_**Questions?**_  
None



##### 10.02.4.1.1. Deep Dive: Summarize, ELI5, Connect

_**Summary**_  
Here we primarily address a _**timing attack**_ where an attacker is able to infer an existing user by checking how long it takes to verify existing vs non-existent users.  
For identifying if a we have valid gmail credentials, this might not be helpful, as there are many other ways to identify valid gmail address. For SaaS application   
a _timing attack_ can help us identify legitimate email credentials that can then be used for other attacks such as phishing as a start.

_timing attacks_ are part of a broader class of _**side-channel**_ attacks, and the ability to confirm if a given user exist or not, we are looking at a potential  
**User Enumeration Vulnerability**.

To be able to expose this vulnerability we write 2 tests
1. `non_existent_users_are_rejected()`
2. `invalid_passwords_are_rejected()`

Then we proceed to add default/placeholder password hash that eliminates timing attacks by ensuring that both non existing credentials and invalid credentials take the  
amount of time to verify.

Lets add the first test
```Rust
//! tests/api/newsletter.rs
// [...]

#[tokio::test]
async fn non_existent_user_is_rejected() {
    // Arrange
    let app = spawn_app().await;
    let placeholder_credentials = Uuid::now_v7().to_string();

    // Act
    let response = reqwest::Client::new()
        .post(format!("{}/newsletters", &app.address))
        .basic_auth(&placeholder_credentials, Some(&placeholder_credentials))
        .json(&serder_json::json!({
            "title": "Newsletter title",
            "content": {
                "plain": "Newsletter as plain text",
                "html": "<p>Newsletter as plain text</p>",
            }
        }))
        .send()
        .await
        .expect("Failed to execute post newsletter request");

    // Assert
    assert_eq!(401, response.status().as_u16());
    assert_eq!(r#"Basic realm="publish""#, response.headers()["WWW-Authenticate"])
}
```

The test should pass. We run the command below to inspect the test logs for how long the validation request takes.
```bash
TEST_LOG=true cargo test --quiet --release non_existent_user_is_rejected | grep "Validate Credentials" | jq -R "fromjson?"
```

We get the below output.

![image.png](10_securing_our_api_files/05aaa560-107d-4e33-ad6f-0930f604156f.png)

The `"time.busy"` is what we are mainly interested in because that is the amount of time spent verifying the credentials, which in this case is about 3.74 milliseconds.

We add the second test `invalid_password_is_rejected`, and inspect the test logs to see how long that takes as well.
```Rust
#[tokio::test]
async fn invalid_password_is_rejected() {
    // Act
    let app = spawn_app().await;
    let username = app.test_user.username;
    let invalid_password = Uuid::now_v7().to_string();
    
    // Arrange
    let response = reqwest::Client::new()
        .post(format!("{}/newsletters", &app.address))
        .basic_auth(username, Some(&invalid_password))
        .json(&serde_json::json!({
            "title": "Newsletter title",
            "content": {
                "plain": "Newsletter as plain text.",
                "html": "<p>Newsletter as plain text.</p>",
            }
        }))
        .send()
        .await
        .expect("Failed to execute newsletter post request in test.");
    
    // Assert
    assert_eq!(401, response.status().as_u16());
    assert_eq!(r#"Basic realm="publish""#, response.headers()["WWW-Authenticate"]);
}
```

When we run the below command on the terminal this time
```bash
TEST_LOGS=true cargo test --quiet --release invalid_password_is_rejected | grep "Validate Credentials" | jq -R "fromjson?"
```
We get;

![image.png](10_securing_our_api_files/758a8b6a-bde4-41e6-bf64-4b4a1959dc9d.png)

The `"time.busy"` is about $10x$ more than the previous test at `28.5` milliseconds, meaning validating invalid credentials is about 10 times slower. What is the best way to address this potential _user enumeration vulnerablity_.
1. Eliminate the time difference between validating non existent credentials and existing ones.
2. Limit number of failed authentication attempts.

While the 2nd option makes generally valuable for a robust protection against brute-force attempts, it requires as keeping track of some state. Let's pause on that for  
now.

The first option is a bit straight forward. Our `validate_credentials` recipe is currently
- Fetch `user_id` and `expected_password` queried by `username`
- If the `username` does not exist we return a `401`
- If they exist we go ahead and verify the stored `expected_password` vs the credentials `password` the user provided on authentication.

This means we are doing an early exist if we don't retrieved stored credentials. We can eliminate this early exit by providing a placeholder PHC string formatted  
password hash and set `user_id` initially to `None`. We then update the values to the actual stored user id and password if the `username` exist, otherwise we use our
placeholder password hash for the verification step. 

This ensures the computationally demanding step of verifying the password will be executed regardless of valid or invalid password.

Lets do this.
```Rust
//! src/routes/newsletter.rs
// [...]

#[tracing::instrument(/**/)]
async fn validate_credentials(/**/) -> Result<Uuid, PublishError> {
    let mut user_id = None;
    let mut expected_password = SecretString::from(
      "$argon2id$v=19$m=19000,t=2,p=1$OqVpaPog6F9sxlWW5VoHkA$4uDo1cl2daKq1ZgmmvtQBfG3wwmI8Nk4i8gHk6pwrYA".to_string()      
    );

    if let Some((stored_user_id, stored_password_hash)) = get_stored_credentials(db_pool, &credentials.username)
        .await
        .map_err(PublishError::Unexpected)?
    {
        user_id = Some(stored_user_id);
        expected_password = stored_password_hash
    };

    spawn_blocking_with_tracing(|| { verify_password_hash( expected_password, &credentials.password) });

    user_id.ok_or_else(|| PublishError::Auth(anyhow::anyhow!("Invalid Username")))
}
```

Now there should be no statistical difference in validation time between existing and non existent credentials

Below is a test log excerpt from the `invalid_password_is_rejected` test.

![image.png](10_securing_our_api_files/86a8e529-9a92-42da-a544-dcee417f2966.png)

And below an excerpt from the `non_existent_user_is_rejected` test.

![image.png](10_securing_our_api_files/57cc59d5-d1ac-47f7-a1c7-c08675d13543.png)

## 10.03. Is It Safe?

##### 10.03.0.0. Skimming: What did you notice and why? Any Questions

_**What?**_  
- Client Credentials via OAuth2
- Session based authenticaiton
- Identity federation that relies on OpenID connect, an identity layer on top of OAuth2 standard.


_**Why?**_  
- Core concepts in Authentication/Authorization and security.
- 

_**Questions?**_  
- Would a detour to understanding how the OAuth2 standard works be worth it?

##### 10.03.0.0.1. Deep Dive: Summarize, ELI5, Connect

_**Summary**_  

When thinking about how secure/safe our basic password-based authentication is, we consider 3 key verticals
- **Transport Layer Security (TLS)** - Currently when using the Basic Authentication Scheme communication between the client and server when passing credentials is encoded
  not encrypted. Using HTTPS ensures that no one can eavesdrop the traffic the client and server and thus be able to decode and compromise the credentials
- **Password Reset** - Incase stored credentials are compromised we need to provision a way for users to reset their password.
- **Interaction Types** - Depending on which interaction types we allow for our backend service/api we need to ensure that appropriate security measures are applied. The common
  interaction types we have to consider are
   - _**Machine to Machine**_ - Our backend service is consumed by another API. We have to be able to authenticate & authorizes requests from such APIs
   - _**Person through a browser interaction**_ - This is the typical client, server interaction whereby a user has some front end interface that exposes our backend service
     functionality. We have to ensure that only valid credentialed users have access to priviledged functionality
   - _**Another API on behalf of person**_ - Primarily for automations that a user wants to enable based on our backend functionality. Imagine another service that a user want to
     use to automate reviewing a published newsletter, that needs to be **scoped** to only review and nothing else. This is different from the first scenario of Machine to Machine
     via API interaction.

How safe is our current implementation for each of the above situations/scenarios?

### 10.03.1. Transport Layer Security (TLS)

Our application is already being served over HTTPS, so we are covered here. There is chance of someone eaves dropping on our traffic and extracting credentials 

### 10.03.2. Password Reset.

We **DO NOT** have a way currently for a user to reset their password. This is something we'll need to add to our implementation

### 10.03.3. Interaction Types.

For interaction types, we boiled them down to
1. Machine To Machine (API)
2. Person Via a Browser
3. Machine to Machine on behalf of a person. (Scoped API)

Lets look at each one by one.

### 10.03.4. Machine To Machine.

#### 10.03.4.0. Overview

One way to do this is where the both services, the consuming and our backend service uses a mutual TLS

#### 10.03.4.1. Client Credentials via OAuth2

Using OAuth2 client credential flow, instead of username and password, we use _client ids_ and _client secrets_ that authenticates with an authorization server  
to generate a temporary Json Web Token (JWT) for example that our backend uses to authenticate/authorize requests. Our **API never sees the actual password**

JWT validation is not without its risk. [Here's](https://www.youtube.com/watch?v=NEqZaHQnqlk) a great resource on hacking JWTs.

### 10.03.5. Person Via Browser.

#### 10.03.5.0. Overview

With our current implementation a user would be required to authenticate on every request. Basically resubmit a username and password to authenticate every request they make  
from the browser. This makes for a very poor user experience. A popular way to handle this is for the server to hold some state around an already authenticated user   
that allows a sequence of request from the same browser without having to resubmit credentials. i.e. remember that this user is authenticated and therefore authorize  
preceding sequence of request from this browser. This is accomplished via **sessions**.

This is where the server generates a one-time short lived token on successful user authentication. The token is included as part of other requests. This token is referred to as  
as _**session token**_ usually store in the browser as a secure cookie. An autheticated session tokens are desgined be short-lived to reduce the likelihood that a valid session is  
compromised. A forceful log-out occurs on session token expiry, necessating a re-authentication, which is a better experience that having to reset a password.

This approach is often reffered to as **Session Based Authentication**.

#### 10.03.5.1. Federated Identity

Another alternative is delegating authentication to third party **identity providers** via Social logins like Google or Facebook Sign in. This makes it very  
convinient for our users to use credentials that they are already used to, making for a great user experience. 

Social auth relies on **identity federation** which in turn relies on **OpenID Connect (OIDC)** a comon impleemetation of an identity layer on-top of OAuth2.

### 10.03.6. Machine to machine, on behalf of a person.

Although this is different from the prior Machine to Machine interaction type because actions/permissions, are scoped, the authentication mechanism fit for this is OAuth2.  
A authentication token from a third party server generated via a client id and secret are used to authorize request to our backend. Our backend has no need for  
storing any credentials, just validating that requests are authorized.

## 10.04. Interlude: Next Steps.

##### 10.01.0.0. Skimming: What did you notice and why? Any Questions

_**What?**_  
- Converting our Basic Authentication to using a Login form with session based authentication

_**Why?**_  
- Looks fun

_**Questions?**_  
- Should we commit to askama now or at the end?
  Tempted to do askama now. But it makes sense to explore it later. Or maybe htmx might be a better exploration here.

##### 10.01.0.0.1. Deep Dive: Summarize, ELI5, Connect

Because a person interacting with our service via a browser is our primary target as of now, we will proceed with **Session Based Authentication** which we will build  
from scratch. This is include
- A login form
- A simple bare bones dashboard with
    - password reset form
    - logout.
    
It will give use an opportunity to tackle a few security challengs such as
- XSS
- new concepts such as HMAC tags
- new tooling i.e. flash messages, `actix-session`

## 10.05. Login Form.

### 10.05.1. Serving HTML Pages

##### 10.05.0.0. Skimming: What did you notice and why? Any Questions

_**What?**_  
- [Internet Is Hard](https://internetingishard.netlify.app/) - For indepth introduction to HTML and CSS
- [Common Rust Lifetimes Misconceptions](https://github.com/pretzelhammer/rust-blog/blob/master/posts/common-rust-lifetime-misconceptions.md#common-rust-lifetime-misconceptions) -
  For a good deep dive into Rust Lifetimes
- [Chrome Dev Tools](https://developer.chrome.com/docs/devtools/open/) & [Firefox Dev Tools](https://firefox-source-docs.mozilla.org/devtools-user/index.html) Documentation

_**Why?**_  
- This book has a treasure trove of cool additional material around Rust, web and software engineering in general.
- All the resources are from the footnotes.

_**Questions?**_  
None


##### 10.05.0.0.1. Deep Dive: Summarize, ELI5, Connect

_**Summary**_  

We primarily serve our first well-formed web page by
1. Adding a `home` module with `mod.rs` and `home.html` to `src/routes`. And do the approrpriate module loading.
2. Add simple html to our `home.html`
3. Wire up the html to be served via a `home` handler in `mod.rs` and update the handler to return a html body content type.
4. Update `startup.rs`'s `run` function with the route handler for handler for the `/home` route

```HTML
<!--src/routes/home/home.html-->
<!DOCTYPE html>
<html lang="en">
  <head>
    <meta http-equiv="conten-type" content-type="text/html" charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Newsletter Home</title>
    <link rel="icon" href="data:,">
  </head>
  <body>
    <p>Welcome to our Newsletter.</p>
  </body>
</html>
```

```Rust
//! src/routes/mod.rs
// [...]
mod home;

// [...]
pub use home::*;
```
```Rust
// src/routes/home/mod.rs
use actix_web::{HttpResponse, http::header::ContentType};

pub async fn home() -> HttpResponse {
    HttpResponse::Ok()
        .content_type(ContentType::html())
        .body(include_str!("home.html"))
}
```
```Rust
//! src/startup.rs
//[...]
use crate::routes::{ /**/,  home }

// [...]

fn run(/**/) -> Result<Server, std::io::Error> {
    // [...]
    let server = HttpServer::new( move || {
        App::new()
            // [...]
            .route("/home", web::get().to(home))
            // [...]
    });

    // [...]
}
```

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

![image.png](10_securing_our_api_files/c44ca008-d37b-45a3-b50d-2825a4fae4f4.png)

![image.png](10_securing_our_api_files/30485ade-7f92-4bb2-a34e-1f5bb40d7b6e.png)

The default `form` is to submit the data to the very same page it is being served from (i.e `/login`) using the `GET` verb. This is far from idea because as we can see forms we submit via `GET`  
encodes our input data in clear text as query parameters. Because query parameters are part of the URL they end up being part of the navigation history and are also captured by the logs

![image.png](10_securing_our_api_files/0fd92a70-3f43-42b5-864d-e754b1b14fc5.png)

To change this behavior we add `method` and `action` attribute to the `form` element as follows

```HTML
<!--src/routes/login/login.html-->
<!--[...]-->
    <form method="POST" action="/login">
<!--[...]-->
```

By adding `method="POST"` the input data becomes part of the request body posted to the `/login` endpoint, which is a much safer option.

![image.png](10_securing_our_api_files/1a108c99-567e-4ff5-9420-3f61d5b4f37c.png)

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

![20260710-0740-55.0795612.gif](10_securing_our_api_files/36c43df5-6040-4717-abd7-a0b680598222.gif)

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

![20260710-1135-13.1725375.gif](10_securing_our_api_files/ff343ba3-1880-4f2f-9f3e-3caf10257f89.gif)

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

![20260713-0712-16.1469524.gif](10_securing_our_api_files/773c9484-b172-4eb1-9d4e-d1ba9c2e8a4e.gif)

It works!

#### 10.06.4.3. Cross-Site Scripting (XSS) 

Query Params in the URL are not private, and nothing prevents a user or an attacker from playing with them to alter them to their purposes. For example try the link below.
```
http://localhost:8000/login?error=Your%20account%20has%20been%20locked%2C%20please%20submit%20your%20details%20%3Ca%20href%3D%22https%3A%2F%2Fzero2prod.com%22%3Ehere%3C%2Fa%3E%20to%20resolve%20the%20issue.
```
This is the result you get

![image.png](10_securing_our_api_files/51dc308d-c23b-48c2-8ac2-ae350d180cad.png)

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

![image.png](10_securing_our_api_files/8938a021-2036-4582-890e-4f4d84f19495.png)

However we need a more robust way of ensuring our messages and secure and verifiable by us. This is because nothing is stopping a attacker from changing our error message and adding their phone number or fake business contact info.

![image.png](10_securing_our_api_files/167817f4-ef71-4514-ba01-3d96e7cba63b.png)

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

![20260715-0655-21.8791896.gif](10_securing_our_api_files/8fb2b4ca-1048-4040-8ff6-7c6f1a275842.gif)

And messages that are tampared with would not be displayed.
![20260715-0708-55.9006862.gif](10_securing_our_api_files/328b9ab7-6612-4ef8-8329-511c623e0942.gif)

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

As we are now entering the final iteration of our design, lets capture the desired behavior in a couple of tests.

#### 10.06.4.10. How To Set A Cookie in `actix-web`



#### 10.06.4.11. An Integration Test For Login Failures - Part 2



#### 10.06.4.12. How To Read A Cookie In `actix-web`



#### 10.06.4.13. How To Delete A Cookie In `actix-web`



#### 10.06.4.14. Cookie Security



#### 10.06.4.15. `actix-web-flash-messages`



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


