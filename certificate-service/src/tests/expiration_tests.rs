//! Tests for expiration/status calculation logic (Valid, Expiring Soon, Expired).

use chrono::{Duration, Utc};

/// Certificate status derived from expiration (mirrors frontend getCertificateStatus logic).
fn certificate_status(expiration: chrono::DateTime<Utc>) -> &'static str {
    let now = Utc::now();
    let thirty_days_from_now = now + Duration::days(30);
    if expiration < now {
        "Expired"
    } else if expiration <= thirty_days_from_now {
        "Expiring Soon"
    } else {
        "Valid"
    }
}

#[test]
fn status_expired_when_expiration_in_past() {
    let past = Utc::now() - Duration::days(1);
    assert_eq!(certificate_status(past), "Expired");
}

#[test]
fn status_expiring_soon_when_expires_within_30_days() {
    let in_15_days = Utc::now() + Duration::days(15);
    assert_eq!(certificate_status(in_15_days), "Expiring Soon");
}

#[test]
fn status_expiring_soon_when_expires_exactly_in_30_days() {
    let in_30_days = Utc::now() + Duration::days(30);
    assert_eq!(certificate_status(in_30_days), "Expiring Soon");
}

#[test]
fn status_valid_when_expires_after_30_days() {
    let in_60_days = Utc::now() + Duration::days(60);
    assert_eq!(certificate_status(in_60_days), "Valid");
}
