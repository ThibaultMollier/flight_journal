use rusqlite::Connection;
use anyhow::{Result, bail};

use super::DATABASE_PATH;

#[derive(Clone)]
pub struct WingTable
{
    pub wing_id: u32,
    pub name: String,
    pub info: String,
    pub def: bool,
}

impl WingTable
{
    pub fn create() -> Result<()>{
        let db_conn = Connection::open(DATABASE_PATH)?;

        match db_conn.execute(
            "CREATE TABLE wings (
                wing_id     INTEGER PRIMARY KEY,
                name        TEXT UNIQUE,
                info        TEXT,
                def         BOOLEAN
            );",
            (), // empty list of parameters.
        ){
            Ok(_) => {
                // First creation of table, insert default wing
                db_conn.execute("INSERT INTO wings (wing_id,name,info,def) VALUES (0,'default','',1)",())?;
            },
            Err(e) => {
                //Table already exist don't pannic
                dbg!(e.to_string());
            }
        }
        db_conn.close().unwrap_or_default();
        Ok(())
    }

    pub fn store(wing: WingTable) -> Result<()>
    {
        let db_conn = Connection::open(DATABASE_PATH)?;
        db_conn.execute(
            "INSERT INTO wings (name, info)
                VALUES (?1, ?2)",
                (
                    wing.name.clone(),
                    wing.info,
                ),
            )?;

            //TODO switch default
            if wing.def {
                WingTable::set_default_wing(None,Some(wing.name))?;
            }

        db_conn.close().unwrap_or_default();
        Ok(())
    }

    pub fn get(id: u32) -> Result<WingTable>
    {
        let db_conn = Connection::open(DATABASE_PATH)?;
        let mut stmt: rusqlite::Statement<'_> = db_conn.prepare("SELECT * FROM wings WHERE wing_id=?1")?;

        let wing = stmt
            .query_row([id], |row| {
                Ok(WingTable {
                    wing_id: row.get(0)?,
                    name: row.get(1)?,
                    info: row.get(2)?,
                    def: row.get(3)?,
                })
            })?;

        Ok(wing)
    }

    pub fn delete(condition: String) -> Result<()>
    {
        let mut sql = "DELETE FROM wings WHERE ".to_string();
        sql.push_str(&condition);
        let db_conn = Connection::open(DATABASE_PATH)?;
        db_conn.execute(&sql,())?;

        db_conn.close().unwrap_or_default();

        Ok(())
    }

    pub fn update(set: String, condition: String) -> Result<()>
    {
        let mut sql = "UPDATE wings SET ".to_string();
        sql.push_str(&set);
        sql.push_str(" WHERE ");
        sql.push_str(&condition);
        let db_conn = Connection::open(DATABASE_PATH)?;
        db_conn.execute(&sql,())?;

        db_conn.close().unwrap_or_default();

        Ok(())
    }

    pub fn select(condition: String) -> Result<Vec<WingTable>>
    {
        let db_conn = Connection::open(DATABASE_PATH)?;
        let mut wings: Vec<WingTable> = Vec::new();
        let mut sql = "SELECT * FROM wings WHERE ".to_string();
        sql.push_str(&condition);

        let mut stmt = db_conn.prepare(&sql)?;

        let rows = stmt
            .query_map([], |row| {
                Ok(WingTable {
                    wing_id: row.get(0)?,
                    name: row.get(1)?,
                    info: row.get(2)?,
                    def: row.get(3)?,
                })
            })?;

        for wing in rows {
            if let Ok(w) = wing {
                wings.push(w)
            }
        }

        Ok(wings)
    }

    pub fn select_all() -> Result<Vec<WingTable>>
    {
        WingTable::select("1".to_string())
    }

    pub fn set_default_wing(id:Option<i32>, name: Option<String>) -> Result<()>
    {
        let db_conn = Connection::open(DATABASE_PATH)?;
        db_conn.execute("UPDATE wings SET def=?1 WHERE def=?2", [0,1])?;

        if let Some(i) = id
        {
            db_conn.execute("UPDATE wings SET def=?1 WHERE wing_id=?2", (true,i))?;
        }else if let Some(n) = name{
            db_conn.execute("UPDATE wings SET def=?1 WHERE name=?2", (true,n))?;
        }
        
        Ok(())
    }

    pub fn get_default_wing() -> Result<WingTable>
    {
        let wing = WingTable::select("def=1".to_string())?;
        Ok(wing[0].clone())
    }
}