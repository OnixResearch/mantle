/// Failure from one external remote-build capability.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RemotePortError {
    pub capability: crunch_remote_core::RemoteCapability,
    pub code: String,
    pub retryable: bool,
}

impl RemotePortError {
    #[must_use]
    pub fn new(capability: crunch_remote_core::RemoteCapability, code: impl Into<String>, retryable: bool) -> Self {
        Self {
            capability,
            code: code.into(),
            retryable,
        }
    }
}

impl core::fmt::Display for RemotePortError {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(formatter, "remote {:?} capability failed: {}", self.capability, self.code)
    }
}

pub trait TransportPort {
    fn execute_transport(
        &mut self,
        effect: crunch_remote_core::RemoteEffect,
    ) -> Result<crunch_remote_core::RemoteObservation, RemotePortError>;
}

pub trait AttemptPersistencePort {
    fn execute_attempt(
        &mut self,
        effect: crunch_remote_core::RemoteEffect,
    ) -> Result<crunch_remote_core::RemoteObservation, RemotePortError>;
}

pub trait ExecutorPort {
    fn execute_build(
        &mut self,
        effect: crunch_remote_core::RemoteEffect,
    ) -> Result<crunch_remote_core::RemoteObservation, RemotePortError>;
}

pub trait StoreAdmissionPort {
    fn execute_admission(
        &mut self,
        effect: crunch_remote_core::RemoteEffect,
    ) -> Result<crunch_remote_core::RemoteObservation, RemotePortError>;
}

pub trait CredentialVerificationPort {
    fn execute_credential_check(
        &mut self,
        effect: crunch_remote_core::RemoteEffect,
    ) -> Result<crunch_remote_core::RemoteObservation, RemotePortError>;
}

pub trait ClockObservationPort {
    fn execute_clock_observation(
        &mut self,
        effect: crunch_remote_core::RemoteEffect,
    ) -> Result<crunch_remote_core::RemoteObservation, RemotePortError>;
}

pub trait RandomIdentifierPort {
    fn execute_identifier_generation(
        &mut self,
        effect: crunch_remote_core::RemoteEffect,
    ) -> Result<crunch_remote_core::RemoteObservation, RemotePortError>;
}

pub trait TelemetryPublicationPort {
    fn execute_telemetry_publication(
        &mut self,
        effect: crunch_remote_core::RemoteEffect,
    ) -> Result<crunch_remote_core::RemoteObservation, RemotePortError>;
}

/// Visible composition root for the eight independent capabilities.
pub struct RemotePortSet<'a> {
    pub transport: &'a mut dyn TransportPort,
    pub attempts: &'a mut dyn AttemptPersistencePort,
    pub executor: &'a mut dyn ExecutorPort,
    pub store: &'a mut dyn StoreAdmissionPort,
    pub credentials: &'a mut dyn CredentialVerificationPort,
    pub clock: &'a mut dyn ClockObservationPort,
    pub identifiers: &'a mut dyn RandomIdentifierPort,
    pub telemetry: &'a mut dyn TelemetryPublicationPort,
}
