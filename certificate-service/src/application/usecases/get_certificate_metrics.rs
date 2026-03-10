use crate::application::dto::DashboardMetrics;
use crate::application::errors::application_error::ApplicationError;
use crate::application::repositories::certificate_repository_abstract::CertificateRepositoryAbstract;
use crate::application::usecases::interfaces::AbstractUseCase;
use async_trait::async_trait;
use std::sync::Arc;

/// Number of days from now to consider "expiring soon".
const EXPIRING_SOON_DAYS: u32 = 30;

pub struct GetCertificateMetricsUseCase {
    certificate_repository: Arc<Box<dyn CertificateRepositoryAbstract>>,
}

impl GetCertificateMetricsUseCase {
    pub fn new(certificate_repository: Arc<Box<dyn CertificateRepositoryAbstract>>) -> Self {
        Self {
            certificate_repository,
        }
    }
}

#[derive(Debug)]
pub struct GetCertificateMetricsInput;

#[derive(Debug)]
pub struct GetCertificateMetricsOutput {
    pub metrics: DashboardMetrics,
}

#[async_trait]
impl AbstractUseCase<GetCertificateMetricsOutput, GetCertificateMetricsInput>
    for GetCertificateMetricsUseCase
{
    async fn execute(
        &self,
        _input: Option<GetCertificateMetricsInput>,
    ) -> Result<GetCertificateMetricsOutput, ApplicationError> {
        let (total, expiring_soon) = self
            .certificate_repository
            .get_metrics(EXPIRING_SOON_DAYS)
            .await?;

        Ok(GetCertificateMetricsOutput {
            metrics: DashboardMetrics {
                total,
                expiring_soon,
            },
        })
    }
}
