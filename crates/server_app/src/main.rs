use std::{path::PathBuf, sync::OnceLock};

use axum::{Router, extract::Query, routing::get};
use clap::Parser;
use rand_core::OsRng;
use short_signed_id::ShortSignedId;
use tokio::net::TcpListener;

use serde::Deserialize;

mod error;

static KEY: OnceLock<[u8; 128]> = OnceLock::new();

#[derive(Parser, Debug)]
struct Args {
    #[arg(long, short)]
    key_file: PathBuf,
    #[arg(long, short)]
    bind: String,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = Args::parse();
    let key: [u8; 128] = std::fs::read(args.key_file)?
        .try_into()
        .map_err(|_| anyhow::anyhow!("Key file must be exactly 128 bytes"))?;
    KEY.set(key).unwrap();

    let app = Router::new()
        .route("/new", get(new_id))
        .route("/verify", get(verify_id));
    let listener = TcpListener::bind(args.bind).await.unwrap();
    axum::serve(listener, app).await.unwrap();
    Ok(())
}

#[derive(Deserialize)]
struct NewIdQuery {
    name: String,
}

async fn new_id(Query(params): Query<NewIdQuery>) -> Result<String, error::AppError> {
    let id = ShortSignedId::new(params.name, KEY.get().unwrap(), &mut OsRng)?;
    Ok(id.to_string())
}

#[derive(Deserialize)]
struct VerifyIdQuery {
    id: String,
}

async fn verify_id(Query(params): Query<VerifyIdQuery>) -> Result<String, error::AppError> {
    let id = ShortSignedId::parse(&params.id)?;
    if id.verify(KEY.get().unwrap())? {
        Ok("true".to_string())
    } else {
        Ok("false".to_string())
    }
}
