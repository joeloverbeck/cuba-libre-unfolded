//! What `cuba-tools seat-wait` reads of a held seat's table while agent sessions play at
//! it: whether the table now needs the agent at the development seat. The loop that keeps
//! the table running is the shared `unfolded_seat_client::hosting`.
use serde::Serialize;
use serde_json::Value;

#[derive(Debug, PartialEq, Eq, Serialize)]
pub struct Seen {
    revision: u64,
    /// The handed-over faction whose decision the table waits for.
    awaits: Option<String>,
    /// What holds an agent's choice at the settled table: what the seat opened.
    holding: Vec<String>,
}

impl Seen {
    /// From the seat's answer, when it describes faction seats.
    pub fn of(answer: &Value) -> Option<Self> {
        let seats = answer["table"].get("faction_seats")?;
        let holding = match seats.get("holding")? {
            Value::Null => vec![],
            held => serde_json::from_value(held.clone()).ok()?,
        };
        Some(Self {
            revision: answer["observation"]["revision"].as_u64()?,
            awaits: serde_json::from_value(seats.get("awaits")?.clone()).ok()?,
            holding,
        })
    }

    /// No agent's faction is awaited, or what was opened holds its choice.
    pub fn needs_the_seat(&self) -> bool {
        self.awaits.is_none() || !self.holding.is_empty()
    }
}
