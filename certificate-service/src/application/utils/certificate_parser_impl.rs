//! Application-layer implementation of CertificateParser using x509-parser.
//! Keeps domain free of x509-parser; business validation lives here.
//!
//! Fix (PEM validity/signature): We read validity and signature from the X.509 TBS/signature
//! structure directly. not_before/not_after come from cert.validity() (ASN1Time in UTC);
//! signature_algorithm from cert.signature_algorithm (the certificate's signature algorithm,
//! e.g. sha1WithRSAEncryption). This avoids wrong dates or hardcoded algorithm names.

use crate::domain::certificate_parser::{CertificateParser, ParsedCertificate};
use x509_parser::pem::parse_x509_pem;
use x509_parser::prelude::*;

const PEM_ERROR_MSG: &str =
    "Invalid PEM format. Please upload a valid PEM encoded certificate or key.";

/// Maps low-level PEM/DER parse errors to the generic user-facing message.
fn to_user_message(_context: &str, raw: impl std::fmt::Display) -> String {
    let s = raw.to_string();
    if s.contains("UnexpectedTag") || s.contains("Parsing Error") || s.contains("Der(")
        || s.contains("PEM")
        || s.contains("base64")
        || s.contains("Invalid")
    {
        PEM_ERROR_MSG.to_string()
    } else {
        format!("{} {}", _context, s)
    }
}

/// Extracts the first -----BEGIN CERTIFICATE----- ... -----END CERTIFICATE----- block from
/// PEM content (e.g. cert+key or key+cert). Returns the block as a string for parsing.
fn extract_first_certificate_block(pem: &str) -> Result<String, String> {
    const BEGIN: &str = "-----BEGIN CERTIFICATE-----";
    const END: &str = "-----END CERTIFICATE-----";
    let trimmed = pem.trim();
    let begin_idx = trimmed.find(BEGIN).ok_or(PEM_ERROR_MSG)?;
    let after_begin = begin_idx + BEGIN.len();
    let end_idx = trimmed[after_begin..].find(END).ok_or(PEM_ERROR_MSG)?;
    let end_idx = after_begin + end_idx + END.len();
    Ok(trimmed[begin_idx..end_idx].to_string())
}

/// PEM → ParsedCertificate using x509-parser (implementation of domain trait).
#[derive(Debug, Default)]
pub struct X509CertificateParser;

impl CertificateParser for X509CertificateParser {
    fn parse_pem(&self, pem: &str) -> Result<ParsedCertificate, String> {
        let cert_block = extract_first_certificate_block(pem)?;
        let (_, pem) = parse_x509_pem(cert_block.trim().as_bytes())
            .map_err(|e| to_user_message("Invalid PEM certificate.", e))?;
        self.parse_der(&pem.contents)
    }
}

impl X509CertificateParser {
    /// Parse DER bytes (e.g. from PEM contents).
    /// Extracts subject, issuer, validity (not_before → valid_from, not_after → expiration),
    /// signature algorithm from the certificate, and SANs. All times are converted to UTC
    /// using ASN1Time::timestamp() (seconds since Unix epoch in UTC).
    pub fn parse_der(&self, der: &[u8]) -> Result<ParsedCertificate, String> {
        let (_, cert) = X509Certificate::from_der(der)
            .map_err(|e| to_user_message("Invalid PEM certificate.", e))?;

        let subject = cert.subject().to_string();
        let issuer = cert.issuer().to_string();

        // Validity: use not_before and not_after from the certificate's Validity structure.
        // ASN1Time::timestamp() returns seconds since 1970-01-01 00:00:00 UTC (RFC 5280).
        let validity = cert.validity();
        let valid_from = chrono::DateTime::from_timestamp(validity.not_before.timestamp(), 0)
            .ok_or_else(|| "Invalid certificate notBefore".to_string())?;
        let expiration = chrono::DateTime::from_timestamp(validity.not_after.timestamp(), 0)
            .ok_or_else(|| "Invalid certificate notAfter".to_string())?;

        // Signature algorithm: from the certificate's signatureAlgorithm field (not TBS).
        // AlgorithmIdentifier has no Display; use OID string (e.g. 1.2.840.113549.1.1.5 = sha1WithRSAEncryption).
        let signature_algorithm = format!("{}", cert.signature_algorithm.algorithm);

        let mut san_entries = Vec::new();
        match cert.subject_alternative_name() {
            Ok(Some(san)) => {
                for name in san.value.general_names.iter() {
                    match name {
                        GeneralName::DNSName(s) => san_entries.push((*s).to_string()),
                        GeneralName::IPAddress(ip) => {
                            let s = if ip.len() == 4 {
                                format!("{}.{}.{}.{}", ip[0], ip[1], ip[2], ip[3])
                            } else if ip.len() == 16 {
                                format!(
                                    "{:02x}{:02x}:{:02x}{:02x}:{:02x}{:02x}:{:02x}{:02x}:{:02x}{:02x}:{:02x}{:02x}:{:02x}{:02x}:{:02x}{:02x}",
                                    ip[0], ip[1], ip[2], ip[3], ip[4], ip[5], ip[6], ip[7],
                                    ip[8], ip[9], ip[10], ip[11], ip[12], ip[13], ip[14], ip[15]
                                )
                            } else {
                                continue;
                            };
                            san_entries.push(s);
                        }
                        _ => {}
                    }
                }
            }
            Ok(None) => {}
            Err(_) => {
                // SAN extension missing or invalid: return empty list (safe fallback; do not break UI).
            }
        }

        Ok(ParsedCertificate {
            subject,
            issuer,
            valid_from,
            expiration,
            signature_algorithm,
            san_entries,
        })
    }
}
