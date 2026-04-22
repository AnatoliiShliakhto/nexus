use super::claims::{DpopClaims, Jkt};
use crate::features::identity::error::IdentityError;
use axum::http::Method;
use base64::{Engine as _, engine::general_purpose::URL_SAFE_NO_PAD};
use jsonwebtoken::jwk::{AlgorithmParameters, EllipticCurve, Jwk};
use jsonwebtoken::{Algorithm, DecodingKey, Validation, decode, decode_header};
use moka::future::Cache;
use redis::aio::ConnectionManager;
use sha2::{Digest, Sha256};
use std::time::Duration;
use smol_str::SmolStr;
use tracing::error;

#[derive(Debug, Clone)]
pub(in crate::features::identity) struct DpopValidator {
    /// Local fallback cache for replay protection if Redis is not provided
    local_cache: Cache<SmolStr, ()>,
    /// Thread-safe, auto-reconnecting Redis handle.
    redis: Option<ConnectionManager>,
    /// Acceptable time drift (in seconds) for `DPoP` timestamps
    time_window_sec: i64,
}

impl DpopValidator {
    pub(in crate::features::identity) fn new(
        time_window_sec: i64,
        max_local_capacity: u64,
        redis: Option<ConnectionManager>,
    ) -> Self {
        let local_cache = Cache::builder()
            .time_to_live(Duration::from_secs(time_window_sec.cast_unsigned() * 2))
            .max_capacity(max_local_capacity)
            .build();

        Self { local_cache, redis, time_window_sec }
    }

    /// Verifies the `DPoP` proof. Accepts an optional multiplexed Redis connection
    /// for distributed replay protection.
    pub(in crate::features::identity) async fn verify(
        &self,
        method: &Method,
        htu: &str,
        proof: &str,
        access_token: Option<&str>,
    ) -> Result<Jkt, IdentityError> {
        let now = chrono::Utc::now().timestamp();

        let header = decode_header(proof)
            .map_err(|e| IdentityError::proof_invalid().with_details(e.to_string()))?;

        if header.alg != Algorithm::EdDSA && header.alg != Algorithm::ES256 {
            return Err(IdentityError::unsupported_key_type());
        }

        let jwk = header.jwk.as_ref().ok_or_else(IdentityError::proof_invalid)?;
        let decoding_key = DecodingKey::from_jwk(jwk)
            .map_err(|e| IdentityError::unsupported_key_type().with_details(e.to_string()))?;

        let jkt = Self::compute_jkt(jwk)?;

        let mut validation = Validation::new(header.alg);
        validation.validate_exp = false;
        validation.required_spec_claims.clear();

        let token_data = decode::<DpopClaims>(proof, &decoding_key, &validation)
            .map_err(|e| IdentityError::verification_failed().with_details(e.to_string()))?;
        let claims = token_data.claims;

        self.validate_time_window(claims.iat, now)?;
        self.validate_context(&claims, method, htu)?;
        self.validate_ath(access_token, claims.ath.as_deref())?;
        self.check_replay(&claims.jti).await?;

        tracing::debug!(jti = %claims.jti, jkt = %jkt, "DPoP proof verified");

        Ok(Jkt::new(jkt))
    }

    fn validate_time_window(&self, iat: i64, now: i64) -> Result<(), IdentityError> {
        if iat > now + 2 || iat < now - self.time_window_sec {
            return Err(IdentityError::time_window_exceeded());
        }
        Ok(())
    }

    fn validate_context(
        &self,
        claims: &DpopClaims,
        method: &Method,
        htu: &str,
    ) -> Result<(), IdentityError> {
        if !claims.htm.eq_ignore_ascii_case(method.as_str()) {
            return Err(IdentityError::verification_failed());
        }

        if &*claims.htu != htu {
            return Err(IdentityError::verification_failed());
        }

        Ok(())
    }

    /// ATH validation.
    fn validate_ath(&self, token: Option<&str>, ath: Option<&str>) -> Result<(), IdentityError> {
        match (token, ath) {
            (Some(at), Some(provided_ath)) => {
                let mut hasher = Sha256::new();
                hasher.update(at.as_bytes());
                let computed_ath = URL_SAFE_NO_PAD.encode(hasher.finalize());

                if computed_ath != provided_ath {
                    return Err(IdentityError::invalid_token_hash());
                }
            },
            (None, Some(_)) => return Err(IdentityError::verification_failed()),
            _ => {},
        }
        Ok(())
    }

    async fn check_replay(&self, jti: &str) -> Result<(), IdentityError> {
        let mut fallback = true;

        if let Some(ref mgr) = self.redis {
            let mut conn = mgr.clone();
            let mut key = String::with_capacity(9 + jti.len());
            key.push_str("dpop:jti:");
            key.push_str(jti);

            let res: Result<Option<String>, redis::RedisError> = redis::cmd("SET")
                .arg(&key)
                .arg("")
                .arg("NX")
                .arg("EX")
                .arg(self.time_window_sec)
                .query_async(&mut conn)
                .await;

            match res {
                Ok(Some(_)) => {
                    fallback = false;
                },
                Ok(None) => {
                    return Err(IdentityError::replay_detected());
                },
                Err(e) => {
                    error!(details = %e, "DPoP: Redis failed, falling back to local cache");
                    fallback = true;
                },
            }
        }

        if fallback {
            let is_new = {
                let mut created = false;
                self.local_cache.get_with(SmolStr::new(jti), async {
                    created = true;
                    ()
                }).await;
                created
            };

            if !is_new {
                return Err(IdentityError::replay_detected());
            }
        }

        Ok(())
    }

    /// JWK Thumbprint computation.
    /// Directly streams strictly ordered JSON bytes into the SHA-256 hasher.
    fn compute_jkt(jwk: &Jwk) -> Result<String, IdentityError> {
        let mut hasher = Sha256::new();

        match &jwk.algorithm {
            AlgorithmParameters::OctetKeyPair(okp) if okp.curve == EllipticCurve::Ed25519 => {
                // {"crv":"Ed25519","kty":"OKP","x":"<x>"}
                hasher.update(b"{\"crv\":\"Ed25519\",\"kty\":\"OKP\",\"x\":\"");
                hasher.update(okp.x.as_bytes());
                hasher.update(b"\"}");
            },
            AlgorithmParameters::EllipticCurve(ec) if ec.curve == EllipticCurve::P256 => {
                // {"crv":"P-256","kty":"EC","x":"<x>","y":"<y>"}
                hasher.update(b"{\"crv\":\"P-256\",\"kty\":\"EC\",\"x\":\"");
                hasher.update(ec.x.as_bytes());
                hasher.update(b"\",\"y\":\"");
                hasher.update(ec.y.as_bytes());
                hasher.update(b"\"}");
            },
            _ => return Err(IdentityError::unsupported_key_type()),
        }

        Ok(URL_SAFE_NO_PAD.encode(hasher.finalize()))
    }
}
