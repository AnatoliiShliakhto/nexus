### Gateway Errors (gw-*)

# Infrastructure & System
gw-infra-redis-error = We are experiencing temporary technical difficulties. Please try again later.
gw-sys-shutdown-init-failed = The service is currently undergoing maintenance. Please try again shortly.
gw-sys-internal-error = An unexpected error occurred on our end. Please try again later.

# Traffic & Routing (Masking upstream/proxy details)
gw-traffic-upstream-timeout = The service is taking too long to respond. Please try again.
gw-traffic-overloaded = Our servers are currently handling a high volume of requests. Please wait a moment and try again.
gw-proxy-target-invalid = We couldn't connect to the requested service right now. Please try again later.
gw-proxy-uri-parse-failed = We couldn't process your request route. Please check the link and try again.
gw-proxy-target-missing = The requested service is currently unavailable. Please try again later.
gw-proxy-request-build-failed = We encountered an issue while processing your request. Please try again.
gw-proxy-dispatch-failed = We are unable to reach the destination service. Please try again later.

# Access & Validation
gw-auth-access-denied = You do not have permission to access this resource.
gw-payload-invalid-format = The information provided is in an invalid format. Please check your input and try again.


### Identity & Security Errors (identity-*)

# Standard Authentication
identity-invalid-credentials = Invalid username or password. Please try again.
identity-account-blocked = This account has been blocked or disabled. Please contact support for assistance.
identity-ip-restricted = Access from your current location or network is restricted.
identity-authorization-missing = Please log in to access this feature.

# Session & Tokens
identity-refresh-token-missing = Your session has expired. Please log in again to continue.
identity-session-data-corruption = There was an issue verifying your session data. Please log in again.
identity-session-unauthorized = Your session is invalid or has expired. Please log in again.
identity-internal-error = An unexpected security verification error occurred. Please try again later.

# DPoP (Demonstrating Proof-of-Possession)
identity-dpop-missing = Your secure session could not be verified. Please log in again.
identity-dpop-invalid = Your secure session format is invalid. Please log in again.
identity-dpop-verification-failed = For your security, we could not verify this request. Please log in again.
identity-dpop-expired = Your security token has expired. Please refresh the page or try again.
identity-dpop-replay-detected = We detected an unusual duplicate request. For your security, please try again.
identity-dpop-ath-mismatch = Your access token validation failed. Please log in again.
identity-dpop-unsupported-key = Your secure session uses an unsupported format. Please log in again.
identity-dpop-nonce-required = Your secure session needs to be refreshed. Please resubmit your request.