use crate::application::errors::application_error::ApplicationError;
use crate::application::repositories::certificate_repository_abstract::CertificateRepositoryAbstract;
use crate::application::usecases::interfaces::AbstractUseCase;
use async_trait::async_trait;
use std::sync::Arc;

pub struct GetCertificatePemUseCase {
    certificate_repository: Arc<Box<dyn CertificateRepositoryAbstract>>,
}

impl GetCertificatePemUseCase {
    pub fn new(certificate_repository: Arc<Box<dyn CertificateRepositoryAbstract>>) -> Self {
        Self {
            certificate_repository,
        }
    }
}

#[derive(Debug)]
pub struct GetCertificatePemInput {
    pub id: String,
}

#[derive(Debug)]
pub struct GetCertificatePemOutput {
    pub pem: String,
}

#[async_trait]
impl AbstractUseCase<GetCertificatePemOutput, GetCertificatePemInput> for GetCertificatePemUseCase {
    async fn execute(
        &self,
        input: Option<GetCertificatePemInput>,
    ) -> Result<GetCertificatePemOutput, ApplicationError> {
        let input = input.ok_or_else(|| {
            ApplicationError::into::<std::io::Error>(
                None,
                Some("Input is required".to_string()),
                None,
                None,
            )
        })?;

        let pem = self
            .certificate_repository
            .get_pem_by_id(&input.id)
            .await?
            .ok_or_else(|| {
                ApplicationError::into::<std::io::Error>(
                    None,
                    Some(format!("Certificate PEM with id {} not found", input.id)),
                    Some(404),
                    None,
                )
            })?;

        Ok(GetCertificatePemOutput { pem })
    }
}
