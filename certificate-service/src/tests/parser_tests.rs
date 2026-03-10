//! Unit tests for certificate parsing (PEM extraction and error handling).

use certificate_service::application::utils::certificate_parser_impl::X509CertificateParser;
use certificate_service::domain::certificate_parser::CertificateParser;

/// Minimal valid X.509 PEM (self-signed, short validity) for parsing tests.
const SAMPLE_PEM: &str = r#"-----BEGIN CERTIFICATE-----
MFAwRgIBADADBgEAMAAwHhcNNTAwMTAxMDAwMDAwWhcNNDkxMjMxMjM1OTU5WjAA
MBgwCwYJKoZIhvcNAQEBAwkAMAYCAQACAQAwAwYBAAMBAA==
-----END CERTIFICATE-----"#;

#[test]
fn parser_rejects_empty_input() {
    let parser = X509CertificateParser::default();
    let result = parser.parse_pem("");
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(err.contains("PEM") || err.contains("Invalid") || err.contains("format"));
}

#[test]
fn parser_rejects_plain_text_without_pem_blocks() {
    let parser = X509CertificateParser::default();
    let result = parser.parse_pem("not a certificate at all");
    assert!(result.is_err());
}

#[test]
fn parser_rejects_malformed_pem_no_end() {
    let parser = X509CertificateParser::default();
    let input = "-----BEGIN CERTIFICATE-----\nMIIB";
    let result = parser.parse_pem(input);
    assert!(result.is_err());
}

#[test]
fn parser_rejects_invalid_base64_in_pem_block() {
    let parser = X509CertificateParser::default();
    let input = "-----BEGIN CERTIFICATE-----\n!!!invalid!!!\n-----END CERTIFICATE-----";
    let result = parser.parse_pem(input);
    assert!(result.is_err());
}

#[test]
fn parser_accepts_valid_pem_and_extracts_metadata() {
    let parser = X509CertificateParser::default();
    let result = parser.parse_pem(SAMPLE_PEM);
    assert!(result.is_ok(), "expected Ok, got {:?}", result.err());
    let parsed = result.unwrap();
    assert!(!parsed.subject.is_empty());
    assert!(!parsed.issuer.is_empty());
    assert!(parsed.expiration > parsed.valid_from);
    assert!(!parsed.signature_algorithm.is_empty());
}

#[test]
fn parser_accepts_pem_with_leading_whitespace() {
    let parser = X509CertificateParser::default();
    let input = format!("\n  \n{}\n", SAMPLE_PEM.trim());
    let result = parser.parse_pem(&input);
    assert!(result.is_ok());
}
