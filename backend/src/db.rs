//! Database module for PostgreSQL interactions

use diesel::prelude::*;
use diesel::r2d2::{self, ConnectionManager};
use std::env;
use log;

pub type DbPool = r2d2::Pool<ConnectionManager<PgConnection>>;

/// Establishes a connection pool to the database
pub fn establish_connection() -> DbPool {
    log::info!("Establishing database connection pool");
    
    let database_url = match env::var("DATABASE_URL") {
        Ok(url) => {
            log::debug!("Database URL loaded successfully");
            url
        }
        Err(_) => {
            log::error!("Environment variable DATABASE_URL is not set");
            panic!("Database configuration error: DATABASE_URL environment variable must be set");
        }
    };

    let manager = ConnectionManager::<PgConnection>::new(database_url);
    
    match r2d2::Pool::builder().build(manager) {
        Ok(pool) => {
            log::info!("Database connection pool established successfully");
            pool
        }
        Err(e) => {
            log::error!("Failed to create database connection pool: {}", e);
            panic!("Database configuration error");
        }
    }
}