use super::schema::Schema;
use crate::Result;
use rusqlite::{Connection, Result as SqliteResult};
use std::path::Path;
use std::sync::{Arc, Mutex};

#[derive(Clone)]
pub struct Database {
    connection: Arc<Mutex<Connection>>,
}

impl Database {
    pub fn new(database_url: &str) -> Result<Self> {
        // Ensure the parent directory exists
        if let Some(parent) = Path::new(database_url).parent() {
            std::fs::create_dir_all(parent)?;
        }

        let conn = Connection::open(database_url)?;
        let database = Database {
            connection: Arc::new(Mutex::new(conn)),
        };

        // Initialize schema
        database.initialize_schema()?;

        Ok(database)
    }

    pub fn execute(&self, sql: &str, params: &[&dyn rusqlite::ToSql]) -> Result<usize> {
        let conn = self.connection.lock().unwrap();
        Ok(conn.execute(sql, params)?)
    }

    fn initialize_schema(&self) -> Result<()> {
        let conn = self.connection.lock().unwrap();
        Schema::initialize(&conn)?;
        Ok(())
    }

    pub fn query_row<T, F>(&self, sql: &str, params: &[&dyn rusqlite::ToSql], f: F) -> Result<T>
    where
        F: FnOnce(&rusqlite::Row) -> SqliteResult<T>,
    {
        let conn = self.connection.lock().unwrap();
        Ok(conn.query_row(sql, params, f)?)
    }

    pub fn query_map<T, F>(
        &self,
        sql: &str,
        params: &[&dyn rusqlite::ToSql],
        mut f: F,
    ) -> Result<Vec<T>>
    where
        F: FnMut(&rusqlite::Row) -> SqliteResult<T>,
    {
        let conn = self.connection.lock().unwrap();
        let mut stmt = conn.prepare(sql)?;
        let rows = stmt.query_map(params, &mut f)?;

        let mut results = Vec::new();
        for row in rows {
            results.push(row?);
        }

        Ok(results)
    }
}
