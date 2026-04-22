use crate::domain::args::DevAction;
use crate::error::AppError;
use crate::services::docker::DockerCompose;

/// Starts or stops the local infrastructure.
///
/// # Result
/// Returns `Ok(())` after executing the requested Docker Compose action.
///
/// # Errors
/// Returns an error if the Docker Compose command fails.
pub(crate) fn handle_dev_command(action: DevAction) -> Result<(), AppError> {
    let docker = DockerCompose::new();

    match action {
        DevAction::Up {} => {
            docker.up()?;
            println!("\n> ✨ Infrastructure is ready.");
        },
        DevAction::Down { volumes } => {
            docker.down(volumes)?;
        },
        DevAction::Logs { service } => {
            docker.logs(service.as_deref())?;
        },
    }

    Ok(())
}
