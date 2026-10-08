//! Positive: a bounded effect admits only its matching typed observation.
//!
//! ```
//! use crunch_remote_core::effect::{EffectEvent, EffectKind, EffectLimits, EffectSession, Observation, ObservedEffect};
//! let mut session = EffectSession::new("request-1").unwrap();
//! let effect = session.plan(EffectKind::OutputAdmission, EffectLimits {
//!     bytes_max: 0, items_max: 1, time_secs_max: 0,
//! }).unwrap();
//! assert_eq!(session.observe(Observation::Succeeded {
//!     id: effect.id, effect: ObservedEffect::OutputsAdmitted { outputs: 1 },
//! }), Ok(EffectEvent::Completed(effect.id)));
//! ```
//!
//! Negative: a host process handle cannot masquerade as a core observation.
//!
//! ```compile_fail
//! use crunch_remote_core::effect::{EffectKind, EffectLimits, EffectSession, Observation};
//! let mut session = EffectSession::new("request-1").unwrap();
//! let effect = session.plan(EffectKind::ExecutorLaunch, EffectLimits {
//!     bytes_max: 0, items_max: 1, time_secs_max: 0,
//! }).unwrap();
//! let _ = session.observe(Observation::Succeeded {
//!     id: effect.id, effect: std::process::Command::new("sh"),
//! });
//! ```
//!
//! Positive: the app owns a capability failure, not a CLI or std error.
//!
//! ```
//! use crunch_remote_app::{CredentialVerificationPort, PortError, verify_credentials};
//! use crunch_remote_core::effect::{EffectKind, EffectLimits, EffectSession};
//! struct Verifier;
//! impl CredentialVerificationPort for Verifier {
//!     fn verify(&mut self, _: &str, _: &[u8]) -> Result<(), PortError> { Ok(()) }
//! }
//! let mut session = EffectSession::new("request-1").unwrap();
//! verify_credentials(&mut session, &mut Verifier, "request-1", b"ticket").unwrap();
//! let next = session.plan(EffectKind::ExecutorLaunch, EffectLimits {
//!     bytes_max: 0, items_max: 1, time_secs_max: 0,
//! }).unwrap();
//! assert_eq!(next.kind, EffectKind::ExecutorLaunch);
//! ```
//!
//! Negative: an application credential port cannot return a host error.
//!
//! ```compile_fail
//! use crunch_remote_app::CredentialVerificationPort;
//! struct HostVerifier;
//! impl CredentialVerificationPort for HostVerifier {
//!     fn verify(&mut self, _: &str, _: &[u8]) -> Result<(), std::io::Error> { Ok(()) }
//! }
//! ```

#[cfg(feature = "host-api")]
pub struct ForbiddenHostApi {
    pub file: std::fs::File,
    pub command: std::process::Command,
    pub network: std::net::TcpStream,
    pub environment: std::env::Vars,
    pub clock: std::time::Instant,
}
