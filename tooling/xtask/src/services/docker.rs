use crate::error::{AppError, AppErrorExt};
use std::path::Path;
use std::process::{Command, Stdio};

#[derive(Debug)]
pub(crate) struct DockerCompose {
    file_path: String,
}

impl Default for DockerCompose {
    fn default() -> Self {
        Self { file_path: "ops/nexus-dev/docker-compose.yml".to_owned() }
    }
}

impl DockerCompose {
    /// Creates a Docker Compose helper with the default `compose` file path.
    ///
    /// # Result
    /// Returns a `DockerCompose` configured for `ops/nexus-dev/docker-compose.yml`.
    ///
    /// # Errors
    /// This function does not return errors.
    #[must_use]
    pub(crate) fn new() -> Self {
        Self::default()
    }

    /// Override the composition file path (useful for testing/custom setups).
    ///
    /// # Result
    /// Returns a `DockerCompose` configured with the provided file path.
    ///
    /// # Errors
    /// This function does not return errors.
    pub(crate) fn with_file_path(path: impl Into<String>) -> Self {
        Self { file_path: path.into() }
    }

    /// Run a docker-compose command
    ///
    /// # Errors
    ///
    pub(crate) fn run(&self, args: &[&str]) -> Result<(), AppError> {
        if !Path::new(&self.file_path).exists() {
            Err(AppError::internal()
                .with_message("Docker compose file not found")
                .with_details_fn(|| format!("File path was `{}`", self.file_path)))?;
        }

        let status = Command::new("docker")
            .arg("compose")
            .arg("-f")
            .arg(&self.file_path)
            .args(args)
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit())
            .status()
            .with_details("Failed to execute docker command")
            .with_help("Is Docker installed and in your PATH?")?;

        if !status.success() {
            Err(AppError::internal().with_details_fn(|| {
                format!("Docker exited with status: {}", status.code().unwrap_or(-1))
            }))?;
        }

        Ok(())
    }

    /// Bring up the infrastructure
    ///
    /// # Result
    /// Returns `Ok(())` after starting the infrastructure.
    ///
    /// # Errors
    /// Returns an error if the Docker Compose command fails.
    pub(crate) fn up(&self) -> Result<(), AppError> {
        println!("🚀 Bringing up infrastructure...");
        self.run(&["up", "-d", "--remove-orphans"])
    }

    /// Shuts down the infrastructure.
    ///
    /// # Result
    /// Returns `Ok(())` after stopping the infrastructure.
    ///
    /// # Errors
    /// Returns an error if the Docker Compose command fails.
    pub(crate) fn down(&self, volumes: bool) -> Result<(), AppError> {
        println!("> 🛑 Shutting down infrastructure...");
        let mut args = vec!["down"];
        if volumes {
            args.push("-v");
        }
        self.run(&args)
    }

    /// Streams logs for a specific service or all services.
    ///
    /// # Result
    /// Returns `Ok(())` after the logs command is started.
    ///
    /// # Errors
    /// Returns an error if the Docker Compose command fails.
    pub(crate) fn logs(&self, service: Option<&str>) -> Result<(), AppError> {
        let mut args = vec!["logs", "-f"];
        if let Some(s) = service {
            args.push(s);
        }
        self.run(&args)
    }
}
