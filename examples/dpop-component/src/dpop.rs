use crate::error::{DpopError, DpopErrorExt};
use base64::{Engine, engine::general_purpose::URL_SAFE_NO_PAD};
use jsonwebtoken::jwk::{AlgorithmParameters, EllipticCurve};
use jsonwebtoken::{Algorithm, DecodingKey, Validation, decode, decode_header};
use nx_dpop_api_lib::{ComputeThumbprintRequest, Jkt, ProofClaims, VerifyRequest};
use nx_http::spin_sdk::http::Request;
use nx_http::spin_sdk::http::body::IncomingBodyExt;
use nx_http::spin_sdk::redis::{Connection, RedisParameter};
use nx_http::spin_tools::environment::get_spin_var;
use nx_http::url::Url;
use sha2::{Digest, Sha256};

/// Main entry point for `DPoP` proof verification.
#[cfg_attr(not(target_arch = "wasm32"), utoipa::path(
    post,
    path = "/verify",
    tag = crate::openapi::TAG,
    operation_id = "dpop_verify_proof",
    summary = "Verify DPoP Proof",
    description = "Validates a `DPoP` proof according to **RFC 9449**.",
    request_body(
        content = VerifyRequest,
        content_type = "application/octet-stream",
        description = "Binary stream of `VerifyRequest` serialized via **Postcard**."
    ),
    responses(
        (
            status = 200,
            content_type = "text/plain",
            body = Jkt,
            description = "Verification successful. Returns the calculated JWK Thumbprint (Jkt).",
        ),
        (
            status = 401,
            description = "Verification failed (e.g., replay detected, invalid signature, or window expired).",
        )
    ),
))]
pub(crate) async fn handle_verify_proof(request: Request) -> Result<Jkt, DpopError> {
    let body = request.body().bytes().await?;
    let payload: VerifyRequest = postcard::from_bytes(&body)?;

    let dpop_window_secs = dpop_window();

    let (claims, header) =
        decode_and_validate_claims(&payload.proof, &payload.method, &payload.uri)?;

    check_replay(&claims.jti, dpop_window_secs)?;

    let key_type = extract_jwk_params(&header)?;
    verify_signature(&payload.proof, &key_type)?;
    validate_ath(payload.access_token, claims.ath.as_deref())?;

    let calculated_jkt = generate_jkt(&key_type);

    if let Some(expected_jkt) = claims.cnf.as_ref().map(|c| &c.jkt)
        && &calculated_jkt != expected_jkt
    {
        return Err(DpopError::verification_failed().with_details("cnf jkt mismatch"));
    }

    Ok(Jkt(calculated_jkt))
}

/// Compute the thumbprint for session binding.
#[cfg_attr(not(target_arch = "wasm32"), utoipa::path(
    post,
    path = "/compute",
    tag = crate::openapi::TAG,
    operation_id = "dpop_compute_thumbprint",
    summary = "Compute JWK Thumbprint",
    description = "Extracts the JWK from the proof header and calculates its **SHA-256 thumbprint** (RFC 7638).",
    request_body(
        content = ComputeThumbprintRequest,
        content_type = "application/octet-stream",
        description = "Binary stream of `ComputeThumbprintRequest` serialized via **Postcard**.",
    ),
    responses(
        (
            status = 200,
            content_type = "text/plain",
            body = Jkt,
            description = "Returns the base64-url encoded JWK Thumbprint string.",
        ),
        (
            status = 400,
            description = "Invalid proof format or unsupported elliptic curve.",
        )
    ),
))]
pub(crate) async fn handle_compute_thumbprint(request: Request) -> Result<Jkt, DpopError> {
    let body = request.body().bytes().await?;
    let payload: ComputeThumbprintRequest = postcard::from_bytes(&body)?;
    let header = decode_header(&payload.proof)
        .map_err(|e| DpopError::proof_invalid().with_details(e.to_string()))?;

    let key_type = extract_jwk_params(&header)?;
    Ok(Jkt(generate_jkt(&key_type)))
}

// --- Shared Internal Helpers ---

/// Validates the structure, claims, time-window, and HTTP context of the proof.
fn decode_and_validate_claims(
    proof: &str,
    method: &str,
    uri: &str,
) -> Result<(ProofClaims, jsonwebtoken::Header), DpopError> {
    let header =
        decode_header(proof).map_err(|e| DpopError::proof_invalid().with_details(e.to_string()))?;

    let parts: Vec<&str> = proof.split('.').collect();
    if parts.len() != 3 {
        return Err(DpopError::proof_invalid().with_details("Invalid structure"));
    }

    let payload_bytes = URL_SAFE_NO_PAD
        .decode(parts[1])
        .map_err(|_| DpopError::proof_invalid().with_details("Invalid base64"))?;

    let claims = serde_json::from_slice::<ProofClaims>(&payload_bytes)
        .map_err(|_| DpopError::proof_invalid().with_details("Invalid claims"))?;

    // Time window validation
    let now = chrono::Utc::now().timestamp();
    let window = dpop_window();
    if claims.iat > now + window || claims.iat < now - window {
        return Err(DpopError::verification_failed().with_details_fn(|| {
            format!("iat out of window: iat={}, now={now}, window={window}", claims.iat)
        }));
    }

    // Context (HTM/HTU) validation
    if !claims.htm.eq_ignore_ascii_case(method) {
        return Err(DpopError::verification_failed()
            .with_details_fn(|| format!("htm mismatch: expected {method}, got {}", claims.htm)));
    }

    let p_uri = Url::parse(uri).with_details_fn(|| format!("Invalid URI: {uri}"))?;
    let p_htu =
        Url::parse(&claims.htu).with_details_fn(|| format!("Invalid htu: {}", claims.htu))?;
    if p_uri.origin() != p_htu.origin() || p_uri.path() != p_htu.path() {
        return Err(DpopError::verification_failed()
            .with_details_fn(|| format!("htu mismatch: expected {p_htu}, got {p_uri}")));
    }

    Ok((claims, header))
}

/// Checks Redis for JTI (JWT ID) to prevent replay attacks.
fn check_replay(jti: &str, window: i64) -> Result<(), DpopError> {
    let redis_url = get_spin_var("redis_url", None)?;
    let conn = Connection::open(&redis_url)?;
    let key = redis_key(jti);

    let res = conn.execute(
        "SET",
        &[
            RedisParameter::Binary(key.as_bytes().to_vec()),
            RedisParameter::Binary(b"1".to_vec()),
            RedisParameter::Binary(b"NX".to_vec()),
            RedisParameter::Binary(b"EX".to_vec()),
            RedisParameter::Binary((window * 2).to_string().into_bytes()),
        ],
    )?;

    match res.first() {
        Some(nx_http::spin_sdk::redis::RedisResult::Nil) => {
            Err(DpopError::verification_failed().with_details("Replay detected"))
        },
        _ => Ok(()),
    }
}

enum JwkKeyType {
    Ed25519 { x: String },
    P256 { x: String, y: String },
}

/// Safely extracts 'jwk' params from the JWK header.
fn extract_jwk_params(header: &jsonwebtoken::Header) -> Result<JwkKeyType, DpopError> {
    let jwk = header
        .jwk
        .as_ref()
        .ok_or_else(|| DpopError::proof_invalid().with_details("Missing JWK"))?;

    match &jwk.algorithm {
        AlgorithmParameters::OctetKeyPair(params) if params.curve == EllipticCurve::Ed25519 => {
            Ok(JwkKeyType::Ed25519 { x: params.x.clone() })
        },
        AlgorithmParameters::EllipticCurve(params) if params.curve == EllipticCurve::P256 => {
            Ok(JwkKeyType::P256 { x: params.x.clone(), y: params.y.clone() })
        },
        _ => Err(DpopError::verification_failed().with_details("Unsupported curve")),
    }
}

/// Verifies the signature of the JWT proof.
fn verify_signature(proof: &str, key_type: &JwkKeyType) -> Result<(), DpopError> {
    let (key, alg) = match key_type {
        JwkKeyType::Ed25519 { x } => {
            let bytes = URL_SAFE_NO_PAD.decode(x).map_err(|_| DpopError::proof_invalid())?;
            (
                DecodingKey::from_ed_components(std::str::from_utf8(&bytes).unwrap())
                    .map_err(|_| DpopError::proof_invalid())?,
                Algorithm::EdDSA,
            )
        },
        JwkKeyType::P256 { x, y } => (
            DecodingKey::from_ec_components(x, y).map_err(|_| DpopError::proof_invalid())?,
            Algorithm::ES256,
        ),
    };

    let mut validation = Validation::new(alg);
    validation.validate_exp = false;
    decode::<ProofClaims>(proof, &key, &validation)
        .map_err(|e| DpopError::verification_failed().with_details(e.to_string()))?;
    Ok(())
}

/// Validates that the Proof is bound to the correct Access Token Hash (ATH).
fn validate_ath(token: Option<String>, ath: Option<&str>) -> Result<(), DpopError> {
    match (token, ath) {
        (Some(at), Some(provided)) => {
            let mut h = Sha256::new();
            h.update(at.as_bytes());
            if URL_SAFE_NO_PAD.encode(h.finalize()) != provided {
                return Err(DpopError::verification_failed().with_details("ath mismatch"));
            }
        },
        (None, Some(_)) => return Err(DpopError::verification_failed()),
        _ => {},
    }
    Ok(())
}

/// Generates a JWK Thumbprint (JKT) per RFC 7638.
fn generate_jkt(key_type: &JwkKeyType) -> String {
    let input = match key_type {
        JwkKeyType::Ed25519 { x } => {
            format!("{{\"crv\":\"Ed25519\",\"kty\":\"OKP\",\"x\":\"{x}\"}}")
        },
        JwkKeyType::P256 { x, y } => {
            format!("{{\"crv\":\"P-256\",\"kty\":\"EC\",\"x\":\"{x}\",\"y\":\"{y}\"}}")
        },
    };
    let mut h = Sha256::new();
    h.update(input.as_bytes());
    URL_SAFE_NO_PAD.encode(h.finalize())
}

fn dpop_window() -> i64 {
    std::env::var("DPOP_WINDOW_SEC").map(|v| v.parse().unwrap_or(60)).unwrap_or(60)
}

fn redis_key(jti: &str) -> String {
    format!("dpop:jti:{jti}")
}

// --- Unit Tests ---

#[cfg(test)]
mod tests {
    use super::*;
    use jsonwebtoken::Header;
    use jsonwebtoken::jwk::{
        AlgorithmParameters, EllipticCurveKeyParameters, Jwk, OctetKeyPairParameters,
    };

    #[test]
    fn test_jkt_generation_consistency() {
        let key = JwkKeyType::Ed25519 { x: "mock_x".to_owned() };
        assert_eq!(generate_jkt(&key), generate_jkt(&key));
    }

    #[test]
    fn test_extract_ed25519_params_success() {
        let header = Header {
            jwk: Some(Jwk {
                common: jsonwebtoken::jwk::CommonParameters::default(),
                algorithm: AlgorithmParameters::OctetKeyPair(OctetKeyPairParameters {
                    key_type: jsonwebtoken::jwk::OctetKeyPairType::default(),
                    curve: EllipticCurve::Ed25519,
                    x: "test_x".to_owned(),
                }),
            }),
            ..Default::default()
        };

        let key = extract_jwk_params(&header).unwrap();
        assert!(matches!(key, JwkKeyType::Ed25519 { .. }), "Expected Ed25519 variant");
    }

    #[test]
    fn test_extract_p256_params_success() {
        let header = Header {
            jwk: Some(Jwk {
                common: jsonwebtoken::jwk::CommonParameters::default(),
                algorithm: AlgorithmParameters::EllipticCurve(EllipticCurveKeyParameters {
                    key_type: jsonwebtoken::jwk::EllipticCurveKeyType::default(),
                    curve: EllipticCurve::P256,
                    x: "x_val".to_owned(),
                    y: "y_val".to_owned(),
                }),
            }),
            ..Default::default()
        };

        let key = extract_jwk_params(&header).unwrap();
        if let JwkKeyType::P256 { x, y } = &key {
            assert_eq!(x, "x_val");
            assert_eq!(y, "y_val");
        }
    }

    #[test]
    fn test_ath_validation_cases() {
        let token = "token".to_owned();
        let mut h = Sha256::new();
        h.update(token.as_bytes());
        let valid_ath = URL_SAFE_NO_PAD.encode(h.finalize());

        assert!(validate_ath(Some(token.clone()), Some(&valid_ath)).is_ok());
        assert!(validate_ath(Some(token), Some("wrong")).is_err());
    }
}
