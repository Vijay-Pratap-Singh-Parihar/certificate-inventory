pub mod get_certificate;
pub mod get_certificate_metrics;
pub mod get_certificate_pem;
pub mod interfaces;
pub mod list_certificates;
pub mod save_certificate;
pub mod save_certificate_from_metadata;

use crate::application::repositories::certificate_repository_abstract::CertificateRepositoryAbstract;
use crate::domain::certificate_parser::CertificateParser;
use std::sync::Arc;

pub struct UseCases {
    pub save_certificate: save_certificate::SaveCertificateUseCase,
    pub save_certificate_from_metadata: save_certificate_from_metadata::SaveCertificateFromMetadataUseCase,
    pub get_certificate: get_certificate::GetCertificateUseCase,
    pub get_certificate_pem: get_certificate_pem::GetCertificatePemUseCase,
    pub list_certificates: list_certificates::ListCertificatesUseCase,
    pub get_certificate_metrics: get_certificate_metrics::GetCertificateMetricsUseCase,
}

impl UseCases {
    pub fn new(
        certificate_repository: Arc<Box<dyn CertificateRepositoryAbstract>>,
        certificate_parser: Arc<Box<dyn CertificateParser>>,
    ) -> Self {
        Self {
            save_certificate: save_certificate::SaveCertificateUseCase::new(
                certificate_repository.clone(),
                certificate_parser.clone(),
            ),
            save_certificate_from_metadata: save_certificate_from_metadata::SaveCertificateFromMetadataUseCase::new(
                certificate_repository.clone(),
            ),
            get_certificate: get_certificate::GetCertificateUseCase::new(
                certificate_repository.clone(),
                certificate_parser.clone(),
            ),
            get_certificate_pem: get_certificate_pem::GetCertificatePemUseCase::new(
                certificate_repository.clone(),
            ),
            list_certificates: list_certificates::ListCertificatesUseCase::new(
                certificate_repository.clone(),
            ),
            get_certificate_metrics: get_certificate_metrics::GetCertificateMetricsUseCase::new(
                certificate_repository,
            ),
        }
    }
}
