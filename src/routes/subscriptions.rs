use actix_web::{
    web::{Data, Form},
    HttpResponse, Responder,
};
use chrono::Utc;
use serde::Deserialize;
use sqlx::PgPool;
use tracing::{error, info, instrument};
use uuid::Uuid;

#[derive(Deserialize)]
pub struct FormData {
    name: String,
    email: String,
}

#[instrument(name="Add a new subscriber",skip(form,pool),fields(
    subscriber_email = %form.email,
    subscriber_name = %form.name,
))]
pub async fn subscribe(form: Form<FormData>, pool: Data<PgPool>) -> impl Responder {
    info!(
        "Adding '{}' '{}' as a new subscriber.",
        form.email, form.name
    );
    if insert_subscriber(&pool, &form).await.is_err() {
        return HttpResponse::InternalServerError().finish();
    }
    info!("New subscriber details have been saved");
    HttpResponse::Ok().finish()
}
#[instrument(
    name = "Saving new subscriber detail in the database",
    skip(pool, form)
)]
pub async fn insert_subscriber(pool: &PgPool, form: &FormData) -> Result<(), sqlx::Error> {
    sqlx::query!(
        r#"
        INSERT INTO subscriptions (id, email, name, subscribed_at )
        VALUES ($1, $2, $3, $4) 
        "#,
        Uuid::new_v4(),
        form.email,
        form.name,
        Utc::now()
    )
    .execute(pool)
    .await
    .map_err(|e| {
        error!("Failed to insert query: {:?}", e);
        e
    })?;
    Ok(())
}
