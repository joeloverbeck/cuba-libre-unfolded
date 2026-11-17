//! The brief form of a development seat's reply, which `cuba-tools seat --brief` prints
//! instead of the full line (#91). It keeps what an agent needs to send its next
//! request: the reply's `status`, `revision`, `settled` and `error`, and each rules
//! choice's id, label and availability. It drops the rest of the observation and the
//! whole table, component names included.
use serde::Serialize;
use serde_json::Value;

#[derive(Serialize)]
struct Brief<'a> {
    status: &'a Value,
    #[serde(skip_serializing_if = "Option::is_none")]
    revision: Option<&'a Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    settled: Option<&'a Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    error: Option<&'a Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    choices: Option<Vec<Choice<'a>>>,
}

#[derive(Serialize)]
struct Choice<'a> {
    #[serde(skip_serializing_if = "Option::is_none")]
    id: Option<&'a Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    label: Option<&'a Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    supported: Option<&'a Value>,
    /// Why the choice is unavailable; left out of a supported choice.
    #[serde(skip_serializing_if = "Option::is_none")]
    explanation: Option<&'a Value>,
}

/// One reply line in brief, as one JSON line without its line ending.
pub fn brief(line: &str) -> String {
    let Ok(reply) = serde_json::from_str::<Value>(line) else {
        return line.to_owned();
    };
    let Some(status) = reply.get("status") else {
        return line.to_owned();
    };
    let present = |key| reply.get(key).filter(|value: &&Value| !value.is_null());
    let choices = reply
        .get("observation")
        .and_then(|observation| observation.get("choices"))
        .and_then(Value::as_array)
        .map(|choices| {
            choices
                .iter()
                .map(|choice| {
                    let supported = choice.get("supported");
                    Choice {
                        id: choice.get("id"),
                        label: choice.get("label"),
                        supported,
                        explanation: choice
                            .get("explanation")
                            .filter(|_| supported != Some(&Value::Bool(true))),
                    }
                })
                .collect()
        });
    serde_json::to_string(&Brief {
        status,
        revision: present("revision"),
        settled: present("settled"),
        error: present("error"),
        choices,
    })
    .expect("a brief of parsed JSON serialises")
}

/// A writer that passes each line written to it on in [`brief`]. The seat client writes
/// one reply and then flushes, so a flush also ends a reply that came without a line
/// ending.
pub struct Briefly<W: std::io::Write> {
    output: W,
    pending: Vec<u8>,
}

impl<W: std::io::Write> Briefly<W> {
    pub fn new(output: W) -> Self {
        Self {
            output,
            pending: Vec::new(),
        }
    }

    fn reply(&mut self, line: &[u8]) -> std::io::Result<()> {
        let line = line.strip_suffix(b"\n").unwrap_or(line);
        let line = line.strip_suffix(b"\r").unwrap_or(line);
        match std::str::from_utf8(line) {
            Ok(line) => writeln!(self.output, "{}", brief(line)),
            // No reply is text that is not UTF-8: pass its bytes on as they came.
            Err(_) => {
                self.output.write_all(line)?;
                self.output.write_all(b"\n")
            }
        }
    }
}

impl<W: std::io::Write> std::io::Write for Briefly<W> {
    fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
        self.pending.extend_from_slice(bytes);
        while let Some(end) = self.pending.iter().position(|&byte| byte == b'\n') {
            let line: Vec<u8> = self.pending.drain(..=end).collect();
            self.reply(&line)?;
        }
        Ok(bytes.len())
    }

    fn flush(&mut self) -> std::io::Result<()> {
        if !self.pending.is_empty() {
            let line = std::mem::take(&mut self.pending);
            self.reply(&line)?;
        }
        self.output.flush()
    }
}
