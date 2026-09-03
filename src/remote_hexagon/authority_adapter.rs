use crunch_remote::ClockObservationPort;
use crunch_remote::CredentialVerificationPort;
use crunch_remote::RandomIdentifierPort;
use crunch_remote::RemoteEffect;
use crunch_remote::RemoteObservation;
use crunch_remote::RemotePortError;

pub struct RemoteCredentialVerificationAdapter<F> {
    execute: F,
}

impl<F> RemoteCredentialVerificationAdapter<F> {
    pub fn new(execute: F) -> Self {
        Self { execute }
    }
}

impl<F> CredentialVerificationPort for RemoteCredentialVerificationAdapter<F>
where F: FnMut(RemoteEffect) -> Result<RemoteObservation, RemotePortError>
{
    fn execute_credential_check(&mut self, effect: RemoteEffect) -> Result<RemoteObservation, RemotePortError> {
        (self.execute)(effect)
    }
}

pub struct RemoteClockObservationAdapter<F> {
    execute: F,
}

impl<F> RemoteClockObservationAdapter<F> {
    pub fn new(execute: F) -> Self {
        Self { execute }
    }
}

impl<F> ClockObservationPort for RemoteClockObservationAdapter<F>
where F: FnMut(RemoteEffect) -> Result<RemoteObservation, RemotePortError>
{
    fn execute_clock_observation(&mut self, effect: RemoteEffect) -> Result<RemoteObservation, RemotePortError> {
        (self.execute)(effect)
    }
}

pub struct RemoteRandomIdentifierAdapter<F> {
    execute: F,
}

impl<F> RemoteRandomIdentifierAdapter<F> {
    pub fn new(execute: F) -> Self {
        Self { execute }
    }
}

impl<F> RandomIdentifierPort for RemoteRandomIdentifierAdapter<F>
where F: FnMut(RemoteEffect) -> Result<RemoteObservation, RemotePortError>
{
    fn execute_identifier_generation(&mut self, effect: RemoteEffect) -> Result<RemoteObservation, RemotePortError> {
        (self.execute)(effect)
    }
}
