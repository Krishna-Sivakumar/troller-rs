use rusqlite::Connection;
type Error = Box<dyn std::error::Error + Send + Sync>;

/// helps retrieve SQL schemas & other stuff for structs implementing this trait.
pub trait ORM {
    fn schema() -> &'static str;
}

/// an SQLite db handle with the following schema:
/// progress_clock(namespace TEXT, name TEXT, segments INTEGER, segments_filled INTEGER, creation_time DATETIME, ephemeral BOOL)
pub struct DB {
    connection: Connection,
}

#[derive(Debug)]
pub struct ProgressClock {
    pub namespace: String,
    pub name: String,
    pub segments: u8,
    pub segments_filled: u8,
    pub ephemeral: bool,
    pub color: Option<String>,
}

impl ORM for ProgressClock {
    fn schema() -> &'static str {
        return "
        CREATE TABLE IF NOT EXISTS progress_clock(
            namespace TEXT,
            name TEXT,
            segments INTEGER,
            segments_filled INTEGER,
            creation_time DATETIME DEFAULT CURRENT_TIMESTAMP,
            ephemeral BOOL,
            color TEXT,
            PRIMARY KEY(namespace, name)
        );
        CREATE INDEX IF NOT EXISTS progress_clock_ns ON progress_clock(namespace);
        CREATE INDEX IF NOT EXISTS progress_clock_name ON progress_clock(name);
        ";
    }
}

impl DB {
    pub fn new() -> Result<Self, Error> {
        let connection = Connection::open("./troller.sqlite")?;
        connection.execute_batch(ProgressClock::schema())?;

        let db = DB { connection };

        Ok(db)
    }

    pub fn get_clock<'a>(
        &self,
        namespace: &'a String,
        name: &'a String,
    ) -> Result<ProgressClock, Error> {
        let mut statement = self.connection.prepare(
            "SELECT name, segments, segments_filled, creation_time, ephemeral, color
            FROM progress_clock
            WHERE namespace = ?1
            AND name = ?2
            AND ((ephemeral = 1 and julianday('now') - julianday(creation_time) < 1) OR (ephemeral = 0));
        ")?;

        let clock = statement.query_row(rusqlite::params![&namespace, &name], |row| {
            Ok(ProgressClock {
                namespace: namespace.clone(),
                name: row.get(0)?,
                segments: row.get(1)?,
                segments_filled: row.get(2)?,
                ephemeral: row.get(4)?,
                color: Some(row.get(5)?),
            })
        })?;

        Ok(clock)
    }

    /// Given a namespace (user or guild), returns all available clocks.
    pub fn get_available_clocks<'a>(
        &self,
        namespace: &'a String,
        partial: &'a str,
    ) -> Result<Vec<ProgressClock>, Error> {
        let mut statement = self.connection.prepare(
            "SELECT name, segments, segments_filled, creation_time, ephemeral, color
            FROM progress_clock
            WHERE namespace = ?1
            AND name LIKE ?2
            AND ((ephemeral = 1 and julianday('now') - julianday(creation_time) < 1) OR (ephemeral = 0));
        ")?;
        let mut clocks: Vec<ProgressClock> = Vec::new();
        let clock_iter = statement.query_map(
            rusqlite::params![&namespace, format!("%{partial}%")],
            |row| {
                Ok(ProgressClock {
                    namespace: namespace.clone(),
                    name: row.get(0)?,
                    segments: row.get(1)?,
                    segments_filled: row.get(2)?,
                    ephemeral: row.get(4)?,
                    color: Some(row.get(5)?),
                })
            },
        )?;

        for item in clock_iter {
            clocks.push(item?);
        }

        Ok(clocks)
    }

    pub fn remove_clock<'a>(&self, namespace: &'a String, name: &'a String) -> Result<(), Error> {
        let mut statement = self
            .connection
            .prepare("DELETE FROM progress_clock WHERE namespace = ?1 AND name = ?2;")?;
        statement.execute(rusqlite::params![namespace, name])?;
        Ok(())
    }

    pub fn bump_clock<'a>(
        &self,
        namespace: &'a String,
        name: &'a String,
        count: u8,
    ) -> Result<(), Error> {
        let mut statement = self.connection.prepare(
            "UPDATE progress_clock
                SET segments_filled = MIN(segments_filled + ?1, segments)
                WHERE namespace = ?2 AND name = ?3;
            ",
        )?;
        statement.execute(rusqlite::params![count, namespace, name])?;
        Ok(())
    }

    pub fn save_clock(&self, progress_clock: &ProgressClock) -> Result<usize, Error> {
        let mut statement = self.connection.prepare(
            "INSERT INTO progress_clock
            (namespace, name, segments, segments_filled, ephemeral, color)
            VALUES (?1, ?2, ?3, ?4, ?5, ?6);",
        )?;

        statement
            .execute(rusqlite::params![
                &progress_clock.namespace,
                &progress_clock.name,
                &progress_clock.segments,
                &progress_clock.segments_filled,
                &progress_clock.ephemeral,
                &progress_clock
                    .color
                    .as_ref()
                    .unwrap_or(&String::from("green"))
            ])
            .map_err(|e| e.into())
    }
}
