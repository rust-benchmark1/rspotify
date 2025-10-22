use mongodb::{Client, bson::doc};
use surrealdb::{engine::remote::ws::Ws, opt::auth::Root, Surreal};
use serde_json::json;

/// Replaces specific customer documents in MongoDB using provided keys.
pub async fn mongo_replace_keys(keys: &[String]) -> Result<(), Box<dyn std::error::Error>> {
    let client = Client::with_uri_str("mongodb://localhost:27017").await?;
    let db = client.database("appdb");
    let coll = db.collection::<mongodb::bson::Document>("customers");

    if let Some(k0) = keys.get(0) {
        let filter0 = doc! { "customer_token": k0 };
        let replacement0 = doc! { "customer_token": k0, "updated": true };
        let _ = coll.replace_one(filter0, replacement0).await?;
    }

    if let Some(k1) = keys.get(1) {
        let filter1 = doc! { "customer_token": k1 };
        let replacement1 = doc! { "customer_token": k1, "updated": true };
        //SINK
        let _ = coll.replace_one(filter1, replacement1).await?;
    }

    Ok(())
}

/// Updates a customer record in SurrealDB with the provided payload.
pub async fn surreal_update(payload: &str) -> Result<(), Box<dyn std::error::Error>> {
    let db = Surreal::new::<Ws>("127.0.0.1:8000").await?;
    db.signin(Root {
        username: "root",
        password: "root",
    })
    .await?;
    db.use_ns("test").use_db("appdb").await?;

    let table = "customer";
    let id = "some-id";
    let content = json!({ "payload": payload });

    //SINK
    let _: serde_json::Value = db
        .update((table, id))
        .content(content)
        .await?
        .unwrap_or_default();
    Ok(())
}