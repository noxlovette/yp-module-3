mod blog {
    include!(concat!(env!("OUT_DIR"), "/blog.v1.rs"));
}

pub use blog::*;

/// (De)serializes the RFC3339 timestamps blog-server sends over HTTP as the
/// unix-second `int64` these proto messages use everywhere else (gRPC and
/// the wire format both need to agree on one representation per field).
pub mod timestamp_serde {
    use chrono::{DateTime, Utc};
    use serde::{Deserialize, Deserializer, Serializer};

    pub fn serialize<S>(ts: &i64, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let dt = DateTime::<Utc>::from_timestamp(*ts, 0)
            .ok_or_else(|| serde::ser::Error::custom("invalid timestamp"))?;
        serializer.serialize_str(&dt.to_rfc3339())
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<i64, D::Error>
    where
        D: Deserializer<'de>,
    {
        Ok(DateTime::<Utc>::deserialize(deserializer)?.timestamp())
    }
}
