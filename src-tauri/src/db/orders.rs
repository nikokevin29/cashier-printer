use rusqlite::{Connection, Result, params};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Order {
    pub id: i64,
    pub customer_name: String,
    pub content: String,
    pub created_at: String,
}

pub fn create_order(conn: &Connection, customer_name: &str, content: &str) -> Result<i64> {
    conn.execute(
        "INSERT INTO orders (customer_name, content) VALUES (?1, ?2)",
        params![customer_name, content],
    )?;
    Ok(conn.last_insert_rowid())
}

pub fn get_orders(conn: &Connection) -> Result<Vec<Order>> {
    let mut stmt = conn.prepare(
        "SELECT id, customer_name, content, created_at FROM orders ORDER BY created_at DESC",
    )?;
    let rows = stmt.query_map([], |row| {
        Ok(Order {
            id: row.get(0)?,
            customer_name: row.get(1)?,
            content: row.get(2)?,
            created_at: row.get(3)?,
        })
    })?;
    rows.collect()
}

pub fn get_order(conn: &Connection, id: i64) -> Result<Order> {
    conn.query_row(
        "SELECT id, customer_name, content, created_at FROM orders WHERE id = ?1",
        params![id],
        |row| {
            Ok(Order {
                id: row.get(0)?,
                customer_name: row.get(1)?,
                content: row.get(2)?,
                created_at: row.get(3)?,
            })
        },
    )
}

pub fn update_order(
    conn: &Connection,
    id: i64,
    customer_name: &str,
    content: &str,
) -> Result<()> {
    let affected = conn.execute(
        "UPDATE orders SET customer_name = ?1, content = ?2 WHERE id = ?3",
        params![customer_name, content, id],
    )?;
    if affected == 0 {
        Err(rusqlite::Error::QueryReturnedNoRows)
    } else {
        Ok(())
    }
}

pub fn delete_order(conn: &Connection, id: i64) -> Result<()> {
    let affected = conn.execute("DELETE FROM orders WHERE id = ?1", params![id])?;
    if affected == 0 {
        Err(rusqlite::Error::QueryReturnedNoRows)
    } else {
        Ok(())
    }
}

pub fn delete_orders_older_than(conn: &Connection, days: u32) -> Result<usize> {
    let affected = conn.execute(
        "DELETE FROM orders WHERE created_at < datetime('now', ?1)",
        params![format!("-{} days", days)],
    )?;
    Ok(affected)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn setup() -> Connection {
        let conn = Connection::open_in_memory().unwrap();
        conn.execute_batch(
            "CREATE TABLE orders (
                id            INTEGER PRIMARY KEY AUTOINCREMENT,
                customer_name TEXT NOT NULL,
                content       TEXT NOT NULL,
                created_at    DATETIME DEFAULT CURRENT_TIMESTAMP
            );",
        )
        .unwrap();
        conn
    }

    #[test]
    fn create_and_get_order() {
        let conn = setup();
        let id = create_order(&conn, "Pak Budi", "2 sak beras").unwrap();
        assert!(id > 0);
        let order = get_order(&conn, id).unwrap();
        assert_eq!(order.customer_name, "Pak Budi");
        assert_eq!(order.content, "2 sak beras");
        assert_eq!(order.id, id);
        assert!(!order.created_at.is_empty());
    }

    #[test]
    fn get_order_not_found() {
        let conn = setup();
        let err = get_order(&conn, 9999).unwrap_err();
        assert_eq!(err, rusqlite::Error::QueryReturnedNoRows);
    }

    #[test]
    fn get_orders_empty() {
        let conn = setup();
        let orders = get_orders(&conn).unwrap();
        assert!(orders.is_empty());
    }

    #[test]
    fn get_orders_returns_all_inserted() {
        let conn = setup();
        let id1 = create_order(&conn, "Alpha", "item a").unwrap();
        let id2 = create_order(&conn, "Beta", "item b").unwrap();
        let orders = get_orders(&conn).unwrap();
        assert_eq!(orders.len(), 2);
        // Both IDs must appear (order not asserted — same-second timestamps are tied)
        let ids: Vec<i64> = orders.iter().map(|o| o.id).collect();
        assert!(ids.contains(&id1));
        assert!(ids.contains(&id2));
    }

    #[test]
    fn update_order_success() {
        let conn = setup();
        let id = create_order(&conn, "Old Name", "old content").unwrap();
        update_order(&conn, id, "New Name", "new content").unwrap();
        let order = get_order(&conn, id).unwrap();
        assert_eq!(order.customer_name, "New Name");
        assert_eq!(order.content, "new content");
    }

    #[test]
    fn update_order_not_found() {
        let conn = setup();
        let err = update_order(&conn, 9999, "X", "Y").unwrap_err();
        assert_eq!(err, rusqlite::Error::QueryReturnedNoRows);
    }

    #[test]
    fn delete_order_success() {
        let conn = setup();
        let id = create_order(&conn, "To Delete", "stuff").unwrap();
        delete_order(&conn, id).unwrap();
        let err = get_order(&conn, id).unwrap_err();
        assert_eq!(err, rusqlite::Error::QueryReturnedNoRows);
    }

    #[test]
    fn delete_order_not_found() {
        let conn = setup();
        let err = delete_order(&conn, 9999).unwrap_err();
        assert_eq!(err, rusqlite::Error::QueryReturnedNoRows);
    }

    #[test]
    fn delete_orders_older_than_removes_old() {
        let conn = setup();
        // Insert a row with a timestamp 400 days in the past
        conn.execute(
            "INSERT INTO orders (customer_name, content, created_at) VALUES ('Old', 'stuff', datetime('now', '-400 days'))",
            [],
        )
        .unwrap();
        // Insert a recent row
        create_order(&conn, "Recent", "stuff").unwrap();

        let deleted = delete_orders_older_than(&conn, 365).unwrap();
        assert_eq!(deleted, 1);

        let remaining = get_orders(&conn).unwrap();
        assert_eq!(remaining.len(), 1);
        assert_eq!(remaining[0].customer_name, "Recent");
    }

    #[test]
    fn delete_orders_older_than_keeps_recent() {
        let conn = setup();
        create_order(&conn, "New", "item").unwrap();
        let deleted = delete_orders_older_than(&conn, 365).unwrap();
        assert_eq!(deleted, 0);
        assert_eq!(get_orders(&conn).unwrap().len(), 1);
    }
}
