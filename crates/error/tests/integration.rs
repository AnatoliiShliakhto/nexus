#![allow(unused_crate_dependencies)]

#[cfg(test)]
mod tests {
    use nx_error::prelude::*;

    mod db {
        #[nx_error::error]
        pub enum DatabaseError {
            #[error(message = "Database IO failure", status = 507, source = std::io::Error, code = "DB_IO_ERROR")]
            Io,
            #[error(message = "Connection lost", status = 503, code = "DB_CONN_LOST")]
            ConnectionLost,
            #[error(message = "Entity not found", status = 404, code = "DB_NOT_FOUND")]
            NotFound,
        }
    }

    mod auth {
        #[nx_error::error]
        pub enum AuthError {
            #[error(message = "Token expired", status = 401, code = "AUTH_EXPIRED")]
            Expired,
        }
    }

    #[error]
    enum ServiceError {
        #[transparent(db::DatabaseError)]
        Db,
        #[transparent(auth::AuthError)]
        Auth,
        #[error(message = "IO failure", status = 500, source = std::io::Error)]
        Io,
        #[error(message = "Validation failed", status = 400)]
        Validation,
    }

    mod transparency {
        use super::*;

        #[test]
        fn should_delegate_metadata_to_inner_error() {
            let db_err = db::DatabaseError::not_found();
            let svc_err: ServiceError = db_err.into();

            assert_eq!(svc_err.message(), "Entity not found");
            assert_eq!(svc_err.status(), ErrorStatus::NotFound);
            assert_eq!(svc_err.code(), "DB_NOT_FOUND");
        }

        #[test]
        fn should_preserve_inner_metadata_through_multiple_layers() {
            #[error]
            enum DeepError {
                #[transparent(ServiceError)]
                Svc,
            }

            let raw_io = std::io::Error::new(std::io::ErrorKind::PermissionDenied, "disk full");
            let db_err = db::DatabaseError::from(raw_io);
            let svc_err: ServiceError = db_err.into();
            let deep_err: DeepError = svc_err.into();

            assert_eq!(deep_err.status(), ErrorStatus::InsufficientStorage);
            assert_eq!(deep_err.code(), "DB_IO_ERROR");
        }

        #[test]
        fn should_maintain_report_chain_with_transparent_source() {
            let db_err = db::DatabaseError::connection_lost();
            let svc_err: ServiceError = db_err.into();
            let report = format!("{}", svc_err.report());

            assert!(report.contains("Connection lost"));
            assert!(report.contains("DB_CONN_LOST"));
        }
    }

    mod context {
        use super::*;

        #[test]
        fn should_initialize_with_generated_constructors() {
            let err = ServiceError::validation();
            assert_eq!(err.status().as_u16(), 400);
            assert_eq!(err.code(), "VALIDATION");
        }

        #[test]
        fn should_override_metadata_manually() {
            let auth_err = auth::AuthError::expired();
            let svc_err: ServiceError = auth_err.into();

            let enriched = svc_err.with_message("Session invalid").with_help("Please login again");

            assert_eq!(enriched.status(), ErrorStatus::Unauthorized);
            assert_eq!(enriched.message(), "Session invalid");
        }

        #[test]
        fn should_support_lazy_evaluation_via_closures() {
            let field = "username";
            let err =
                ServiceError::validation().with_message_fn(|| format!("Invalid field: {field}"));

            assert_eq!(err.message(), "Invalid field: username");
        }
    }

    mod conversions {
        use super::*;

        #[test]
        fn should_convert_from_std_io_error() {
            let raw_io = std::io::Error::new(std::io::ErrorKind::PermissionDenied, "denied");
            let svc_err = ServiceError::from(raw_io);
            assert_eq!(svc_err.status().as_u16(), 500);
        }

        #[test]
        fn should_enrich_result_via_extension_trait() {
            let res: Result<(), db::DatabaseError> = Err(db::DatabaseError::connection_lost());
            let enriched: Result<(), ServiceError> = res.with_help("Check network");

            assert!(enriched.is_err());
            assert_eq!(enriched.unwrap_err().status().as_u16(), 503);
        }
    }

    mod formatting {
        use super::*;

        #[test]
        fn should_implement_display_and_debug_correctly() {
            let err = ServiceError::validation();
            assert_eq!(format!("{err}"), "Validation failed");
            assert!(format!("{err:?}").contains("ServiceErrorData"));
        }

        #[cfg(feature = "json")]
        #[test]
        fn should_serialize_to_json_with_full_metadata() {
            let db_err = db::DatabaseError::not_found();
            let svc_err: ServiceError = db_err.into();
            let json = svc_err.to_detailed_json_string();

            assert!(json.contains("\"status\":404"));
            assert!(json.contains("\"code\":\"DB_NOT_FOUND\""));
        }
    }
}
