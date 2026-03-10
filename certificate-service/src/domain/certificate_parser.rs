//! Port for parsing PEM-encoded certificates into domain data.
//! Implementations (e.g. using x509-parser) live in the Application layer so that
//! the Domain layer has zero dependencies on external crates.

use chrono::{DateTime, Utc};

/// Result of parsing a PEM certificate: subject, issuer, validity, signature algorithm, SANs.
/// Used by SaveCertificateUseCase to build a [super::Certificate] entity.
#[derive(Debug, Clone)]
pub struct ParsedCertificate {
    pub subject: String,
    pub issuer: String,
    /// Validity start (notBefore) in UTC.
    pub valid_from: DateTime<Utc>,
    /// Validity end (notAfter) in UTC.
    pub expiration: DateTime<Utc>,
    /// Signature algorithm as reported by the certificate (e.g. sha1WithRSAEncryption).
    pub signature_algorithm: String,
    pub san_entries: Vec<String>,
}

/// Parser trait: PEM string → ParsedCertificate.
/// Implemented in application (e.g. with x509-parser); domain stays pure.
pub trait CertificateParser: Send + Sync {
    /// Parse a PEM-encoded X.509 certificate and return metadata.
    /// Returns an error message (or use a dedicated error type in application) on failure.
    fn parse_pem(&self, pem: &str) -> Result<ParsedCertificate, String>;
}
