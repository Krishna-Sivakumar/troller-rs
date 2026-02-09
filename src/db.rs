use rusqlite::Connection;

type Error = Box<dyn std::error::Error + Send + Sync>;

/// helps retrieve SQL schemas & other stuff for structs implementing this trait.
pub trait ORM {
    fn schema() -> &'static str;
    fn table_name() -> &'static str;
}

/// an SQLite db handle with the following schema:
/// progress_clock(namespace TEXT, name TEXT, segments INTEGER, segments_filled INTEGER, creation_time DATETIME, ephemeral BOOL)
pub struct DB {
    connection: Connection,
}

pub struct ProgressClock {
    pub namespace: String,
    pub name: String,
    pub segments: u8,
    pub segments_filled: u8,
    pub creation_date: u32,
    pub ephemeral: bool,
    pub color: Option<String>,
}

impl ORM for ProgressClock {
    fn schema() -> &'static str {
        return "
        CREATE TABLE IF NOT EXISTS progress_clock(namespace TEXT, name TEXT, segments INTEGER, segments_filled INTEGER, creation_time DATETIME, ephemeral BOOL, color TEXT);
        CREATE INDEX IF NOT EXISTS progress_clock_ns ON progress_clock(namespace);
        CREATE INDEX IF NOT EXISTS progress_clock_name ON progress_clock(name);
        ";
    }

    fn table_name() -> &'static str {
        return "progress_clock";
    }
}

impl DB {
    pub fn new() -> Result<Self, Error> {
        const CREATE_QUERY: &str = "";

        let connection = Connection::open("./troller.sqlite")?;
        connection.execute_batch(ProgressClock::schema())?;

        let db = DB { connection };

        Ok(db)
    }

    /// Given a namespace (user or guild), returns all available clocks.
    pub fn get_available_clocks<'a>(
        &self,
        namespace: String,
        partial: &'a str,
    ) -> Result<Vec<ProgressClock>, Error> {
        let mut statement = self.connection.prepare("SELECT name, segments, segments_filled, creation_time, ephemeral, color FROM progress_clock WHERE namespace = ?1 AND name LIKE %?2%")?;
        let mut clocks: Vec<ProgressClock> = Vec::new();
        let clock_iter = statement.query_map(rusqlite::params![namespace, partial], |row| {
            Ok(ProgressClock {
                namespace: namespace.clone(),
                name: row.get(0)?,
                segments: row.get(1)?,
                segments_filled: row.get(2)?,
                creation_date: row.get(3)?,
                ephemeral: row.get(4)?,
                color: Some(row.get(5)?),
            })
        })?;

        for item in clock_iter {
            clocks.push(item?);
        }

        Ok(clocks)
    }

    /// Given a namespace (user or guild) and clock name, returns the status of a progress clock.
    pub fn get_clock(&self, name: String, namespace: String) -> Result<(u8, u8), Error> {
        todo!();
    }
}
