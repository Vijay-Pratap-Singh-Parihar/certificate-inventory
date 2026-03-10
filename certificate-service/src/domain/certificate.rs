//! Domain entity: Certificate.
//! No dependencies on SQLx, Axum, or x509-parser.
//! `expiration` and `valid_from` serialize as ISO 8601 (RFC 3339) via chrono's serde support.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Certificate {
    pub id: String,
    pub subject: String,
    pub issuer: String,
    /// Validity start (notBefore) in UTC. Present when parsed from PEM; null when loaded from DB without re-parsing.
    pub valid_from: Option<DateTime<Utc>>,
    /// Expiration time (not_after) in UTC. Serializes as ISO 8601 string.
    pub expiration: DateTime<Utc>,
    /// Signature algorithm (e.g. sha1WithRSAEncryption). Present when parsed from PEM; null when loaded from DB without re-parsing.
    pub signature_algorithm: Option<String>,
    /// Subject Alternative Names (DNS, IP, etc.).
    pub san_entries: Vec<String>,
    /// Last time the record was updated (for display in inventory).
    pub last_updated: Option<DateTime<Utc>>,
}

impl Certificate {
    pub fn new(
        id: String,
        subject: String,
        issuer: String,
        valid_from: Option<DateTime<Utc>>,
        expiration: DateTime<Utc>,
        signature_algorithm: Option<String>,
        san_entries: Vec<String>,
        last_updated: Option<DateTime<Utc>>,
    ) -> Self {
        Self {
            id,
            subject,
            issuer,
            valid_from,
            expiration,
            signature_algorithm,
            san_entries,
            last_updated,
        }
    }
}
