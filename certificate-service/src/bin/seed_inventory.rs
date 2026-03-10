//! Seed the certificates table with 50k+ records for performance testing.
//! Run with: cargo run --bin seed_inventory
//! Requires DATABASE_URL in environment.

use chrono::{Duration, Utc};
use sqlx::postgres::PgPoolOptions;
use sqlx::PgPool;
use std::env;
use uuid::Uuid;

const TOTAL_RECORDS: u32 = 50_000;
const BATCH_SIZE: usize = 1_000;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    dotenv::dotenv().ok();
    let database_url = env::var("DATABASE_URL").unwrap_or_else(|_| {
        "postgres://postgres:postgres@localhost:5432/certificate_db".to_string()
    });

    println!("Connecting to database...");
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await?;

    println!("Running migrations...");
    sqlx::migrate!("./migrations").run(&pool).await?;

    println!("Seeding {} certificates in batches of {}...", TOTAL_RECORDS, BATCH_SIZE);
    let start = std::time::Instant::now();

    let issuers = [
        "Let's Encrypt",
        "DigiCert",
        "Sectigo",
        "Internal CA",
        "Corporate PKI",
    ];

    let mut inserted = 0u32;
    let mut batch = Vec::with_capacity(BATCH_SIZE);

    for i in 0..TOTAL_RECORDS {
        let id = Uuid::new_v4().to_string();
        let subject = format!("cert-{:05}.example.com", i + 1);
        let issuer = issuers[i as usize % issuers.len()].to_string();
        // Spread expiration: ~40% valid, ~30% expiring soon, ~30% expired
        let expiration = match (i % 10) as u8 {
            0 | 1 | 2 | 3 => Utc::now() + Duration::days(60 + (i % 90) as i64),
            4 | 5 | 6 => Utc::now() + Duration::days((i % 28) as i64),
            _ => Utc::now() - Duration::days((i % 365) as i64),
        };
        let san_entries: Vec<String> = vec![subject.clone()];

        batch.push((id, subject, issuer, expiration, san_entries));

        if batch.len() >= BATCH_SIZE {
            insert_batch(&pool, &batch).await?;
            inserted += batch.len() as u32;
            batch.clear();
            if inserted % 10_000 == 0 {
                println!("  inserted {}...", inserted);
            }
        }
    }
    if !batch.is_empty() {
        insert_batch(&pool, &batch).await?;
        inserted += batch.len() as u32;
    }

    let elapsed = start.elapsed();
    println!("Done. Inserted {} certificates in {:.2}s", inserted, elapsed.as_secs_f64());
    Ok(())
}

async fn insert_batch(
    pool: &PgPool,
    batch: &[(String, String, String, chrono::DateTime<Utc>, Vec<String>)],
) -> Result<(), sqlx::Error> {
    let mut tx = pool.begin().await?;
    for (id, subject, issuer, expiration, san_entries) in batch {
        sqlx::query(
            r#"
            INSERT INTO certificates (id, subject, issuer, expiration, san_entries)
            VALUES ($1, $2, $3, $4, $5)
            ON CONFLICT (id) DO NOTHING
            "#,
        )
        .bind(id)
        .bind(subject)
        .bind(issuer)
        .bind(expiration)
        .bind(san_entries)
        .execute(&mut *tx)
        .await?;
    }
    tx.commit().await?;
    Ok(())
}
