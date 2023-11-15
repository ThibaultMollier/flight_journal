use rusqlite::Connection;
use anyhow::{Result, bail};

use super::{DATABASE_PATH, IDListe};


pub struct TagTable
{
    tag_id: u32,
    name: String,
}

impl TagTable
{
    pub fn create() -> Result<()>{
        let db_conn = Connection::open(DATABASE_PATH)?;

        db_conn.execute("PRAGMA foreign_keys = ON;",())?;

        db_conn.execute(
            "CREATE TABLE IF NOT EXISTS tags (
                tag_id      INTEGER PRIMARY KEY,
                name        TEXT UNIQUE
            );",
            (), // empty list of parameters.
        )?;

        db_conn.execute(
            "CREATE TABLE IF NOT EXISTS tag_asso (
                asso_id         INTEGER PRIMARY KEY,
                asso_tag_id     INTEGER REFERENCES tags(tag_id),
                asso_flight_id  INTEGER REFERENCES flights(flight_id) 
            );",
            (), // empty list of parameters.
        )?;
        db_conn.close().unwrap_or_default();
        Ok(())
    }

    pub fn store(tag: TagTable) -> Result<()>
    {
        let db_conn = Connection::open(DATABASE_PATH)?;
        db_conn.execute(
            "INSERT INTO tags (name)
                VALUES (?1, ?2)",
                (
                    tag.name,
                ),
            )?;

        db_conn.close().unwrap_or_default();
        Ok(())
    }

    pub fn get(id: u32) -> Result<TagTable>
    {
        let db_conn = Connection::open(DATABASE_PATH)?;
        let mut stmt: rusqlite::Statement<'_> = db_conn.prepare("SELECT * FROM tags WHERE tag_id=?1")?;

        let tag = stmt
            .query_row([id], |row| {
                Ok(TagTable {
                    tag_id: row.get(0)?,
                    name: row.get(1)?,
                })
            })?;

        Ok(tag)
    }

    pub fn delete(condition: String) -> Result<()>
    {
        let mut sql = "DELETE FROM tags WHERE ".to_string();
        sql.push_str(&condition);
        let db_conn = Connection::open(DATABASE_PATH)?;
        db_conn.execute(&sql,())?;

        db_conn.close().unwrap_or_default();

        Ok(())
    }

    pub fn update(set: String, condition: String) -> Result<()>
    {
        let mut sql = "UPDATE tags SET ".to_string();
        sql.push_str(&set);
        sql.push_str(" WHERE ");
        sql.push_str(&condition);
        let db_conn = Connection::open(DATABASE_PATH)?;
        db_conn.execute(&sql,())?;

        db_conn.close().unwrap_or_default();

        Ok(())
    }

    pub fn select(condition: String) -> Result<Vec<TagTable>>
    {
        let db_conn = Connection::open(DATABASE_PATH)?;
        let mut tags: Vec<TagTable> = Vec::new();
        let mut sql = "SELECT * FROM tags WHERE ".to_string();
        sql.push_str(&condition);

        let mut stmt = db_conn.prepare(&sql)?;

        let rows = stmt
            .query_map([], |row| {
                Ok(TagTable {
                    tag_id: row.get(0)?,
                    name: row.get(1)?,
                })
            })?;

        for tag in rows {
            if let Ok(t) = tag {
                tags.push(t)
            }
        }

        Ok(tags)
    }

    pub fn associate(flight_id: u32, tag_id: u32) -> Result<()>
    {
        let db_conn = Connection::open(DATABASE_PATH)?;
        db_conn.execute(
            "INSERT OR IGNORE INTO tag_asso (asso_tag_id, asso_flight_id)
                VALUES (?1, ?2)",
                (
                    tag_id,
                    flight_id,
                ),
            )?;
    
        Ok(())
    }

    pub fn select_all() -> Result<Vec<TagTable>>
    {
        TagTable::select("1".to_string())
    }

    pub fn search(search: String) -> Result<IDListe>
    {
        let tags = TagTable::select("1".to_string())?;
        let mut tag_ids:IDListe = IDListe { list: Vec::new() };

        for tag in tags
        {
            if tag.name.to_lowercase().contains(&search.to_lowercase())
            {
                tag_ids.list.push(tag.tag_id);
            }
        }

        Ok(tag_ids)
    }
}