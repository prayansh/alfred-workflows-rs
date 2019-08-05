pub mod models;
pub mod schema;

use crate::database::models::{Config, NewConfig, NewRepository, Repository};
use crate::database::schema::config::dsl::*;
use crate::database::schema::repositories::dsl::*;
use diesel::prelude::*;
use diesel::{Connection, SqliteConnection};
use failure::Error;
use std::env;

embed_migrations!();

pub struct DbContext {
    conn: SqliteConnection,
}

impl DbContext {
    pub fn new() -> Result<Self, Error> {
        let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
        let conn = SqliteConnection::establish(&database_url)?;
        //            .unwrap_or_else(|_| format_err!("Error connecting to {}", database_url));

        //        let conn = alfred_workflow::open_database_or_else("github", DbContext::create_tables)?;
        Ok(DbContext { conn })
    }

    pub fn run_migrations(&self) -> Result<(), diesel_migrations::RunMigrationsError> {
        embedded_migrations::run(&self.conn)
        //        embedded_migrations::run_with_output(&self.conn, &mut std::io::stdout())
    }

    pub fn delete_repositories(&self) -> QueryResult<usize> {
        diesel::delete(repositories).execute(&self.conn)
    }

    pub fn find_repositories(
        &self,
        repo_name: &str,
        limit: i64,
    ) -> Result<Vec<Repository>, diesel::result::Error> {
        // '_' means any character when using 'LIKE'.
        // This will allow searching by full name or just the words within the name.
        let query = repo_name
            .split_terminator(' ')
            .flat_map(|s| s.split_terminator('_'))
            .flat_map(|s| s.split_terminator('-'))
            .collect::<Vec<&str>>()
            .join("_");

        repositories
            .filter(name.like(format!("{}%", query)))
            .order_by(pushed_at.desc())
            .limit(limit)
            .load::<Repository>(&self.conn)
    }

    pub fn insert_repositories(&self, insert_repositories: &[NewRepository]) -> QueryResult<usize> {
        // TODO: accept Iterator and control the batching
        diesel::insert_into(repositories)
            .values(insert_repositories)
            .execute(&self.conn)
    }

    pub fn optimize(&self) -> Result<(), Error> {
        // since this workflow is READ heavy, let's optimize the SQLite indexes and DB
        self.conn.execute("VACUUM;")?;
        Ok(())
    }

    pub fn set_token(&self, api_token: &str) -> QueryResult<usize> {
        let new_config = NewConfig {
            key: "token",
            value: api_token,
        };
        diesel::insert_into(config)
            .values(new_config)
            .execute(&self.conn)
    }

    pub fn get_token(&self) -> QueryResult<Config> {
        config.filter(key.eq("token")).get_result(&self.conn)
    }
}
