### Repetitive `curl` test
```curl
curl -d "username=lei%20yin&email=lei_yin_loo%40gmail.com" http://127.0.0.1:8000/subscriptions
```

### Project Snapshot
```Rust
// ---------------------------------------------------
// src\main.rs
// ---------------------------------------------------
use sqlx::PgPool;
use std::net::TcpListener;

use zero2prod::{config::get_config, startup::run};

#[actix_web::main]
async fn main() -> Result<(), std::io::Error> {
    // We panic if we can't read configuration.
    let config = get_config().expect("Failed to read configuration");
    let connection_pool = PgPool::connect(&config.db.connection_string())
        .await
        .expect("Failed to connect to Postgres.");
    let address = format!("127.0.0.1:{}", config.app_port);
    let listener = TcpListener::bind(address)
        .unwrap_or_else(|_| panic!("Failed to bind to port: {}", config.app_port));
    run(listener, connection_pool)?.await
}

// ---------------------------------------------------
// src\lib.rs
// ---------------------------------------------------
pub mod config;
pub mod routes;
pub mod startup;

// ---------------------------------------------------
// src\config.rs
// ---------------------------------------------------
#[derive(serde::Deserialize)]
pub struct Settings {
    pub db: DBSettings,
    pub app_port: u16,
}

#[derive(serde::Deserialize)]
pub struct DBSettings {
    pub username: String,
    pub password: String,
    pub port: u16,
    pub host: String,
    pub db_name: String,
}

impl DBSettings {
    pub fn connection_string(&self) -> String {
        format!(
            "postgres://{}:{}@{}:{}/{}",
            self.username, self.password, self.host, self.port, self.db_name
        )
    }
}

pub fn get_config() -> Result<Settings, config::ConfigError> {
    // Intialize our config reader
    let settings = config::Config::builder()
        // Add config values from a file name 'config.yaml'.
        .add_source(config::File::new("config.yaml", config::FileFormat::Yaml))
        .build()?;
    // Try to convert the read config values into our Setting type
    settings.try_deserialize::<Settings>()
}

// ---------------------------------------------------
// src\startup.rs
// ---------------------------------------------------
use crate::routes::{greet, health_check, subscribe};
use actix_web::dev::Server;
use actix_web::{App, HttpServer, web};
use sqlx::PgPool;
use std::net::TcpListener;

pub fn run(
    listener: TcpListener,
    db_pool: PgPool, // New param
) -> Result<Server, std::io::Error> {
    let db_pool = web::Data::new(db_pool);

    let server = HttpServer::new(move || {
        App::new()
            .route("/health_check", web::get().to(health_check))
            .route("/", web::get().to(greet))
            .route("/{name}", web::get().to(greet))
            .route("/subscriptions", web::post().to(subscribe))
            // Registering connection as part of applicaton state
            .app_data(db_pool.clone())
    })
    .listen(listener)?
    .run();

    Ok(server)
}

// ---------------------------------------------------
// src\routes\subscriptions.rs
// ---------------------------------------------------
use actix_web::{HttpResponse, web};
use sqlx::PgPool;

#[derive(serde::Deserialize)]
pub struct FormData {
    pub email: String,
    pub username: String,
}

pub async fn subscribe(
    _form: web::Form<FormData>,
    _pool: web::Data<PgPool>, // Retrieving a connection from App State
) -> HttpResponse {
    HttpResponse::Ok().finish()
}
```
```yaml
#  ---------------------------------------------------
# config.yaml
# ---------------------------------------------------
app_port: 8000
db:
  host: "127.0.0.1"
  port: 5432
  username: "postgres"
  password: "password"
  db_name: "newsletter"



```
```sql
SELECT 'DROP TABLE IF EXISTS ' || quote_ident(tablename) || ' CASCADE;'
FROM pg_tables
WHERE schemaname = 'public'
  AND tablename LIKE 'prefix_%';
```
```sql
DO $$
DECLARE
    r RECORD;
BEGIN
    FOR r IN
        SELECT tablename
        FROM pg_tables
        WHERE schemaname = 'public'
          AND tablename LIKE '019d%'
    LOOP
        EXECUTE 'DROP TABLE IF EXISTS ' || quote_ident(r.tablename) || ' CASCADE';
    END LOOP;
END;
$$;

```
```bash
psql -U postgres -t -A -c \
  "SELECT datname FROM pg_database WHERE datname LIKE '019d%';" \
| xargs -I {} psql -U postgres -d postgres -c "DROP DATABASE IF EXISTS \"{}\";"

```
