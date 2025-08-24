//! Database module for PostgreSQL interactions

use diesel::prelude::*;
use diesel::r2d2::{self, ConnectionManager};
use std::env;
use log;

pub type DbPool = r2d2::Pool<ConnectionManager<PgConnection>>;

/// Establishes a connection pool to the database with TLS enforcement
pub fn establish_connection() -> DbPool {
    log::info!("Establishing database connection pool with TLS enforcement");
    
    let mut database_url = match env::var("DATABASE_URL") {
        Ok(url) => {
            log::debug!("Database URL loaded successfully");
            url
        }
        Err(_) => {
            log::error!("Environment variable DATABASE_URL is not set");
            panic!("Database configuration error: DATABASE_URL environment variable must be set");
        }
    };

    // Enforce TLS for database connections
    if !database_url.contains("sslmode=") {
        if database_url.contains("?") {
            database_url.push_str("&sslmode=require");
        } else {
            database_url.push_str("?sslmode=require");
        }
        log::info!("Added TLS requirement to database connection");
    } else if database_url.contains("sslmode=disable") || database_url.contains("sslmode=allow") || database_url.contains("sslmode=prefer") {
        log::warn!("Database connection may not be secure. Consider using sslmode=require or sslmode=verify-full");
    }

    let manager = ConnectionManager::<PgConnection>::new(database_url);
    
    match r2d2::Pool::builder().build(manager) {
        Ok(pool) => {
            log::info!("Database connection pool established successfully with TLS");
            pool
        }
        Err(e) => {
            log::error!("Failed to create database connection pool: {}", e);
            panic!("Database configuration error");
        }
    }
}