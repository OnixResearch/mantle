use crunch_remote::RemoteEffect;
use crunch_remote::RemoteObservation;
use crunch_remote::RemotePortError;
use crunch_remote::TelemetryPublicationPort;

pub struct RemoteTelemetryPublicationAdapter<F> {
    execute: F,
}

impl<F> RemoteTelemetryPublicationAdapter<F> {
    pub fn new(execute: F) -> Self {
        Self { execute }
    }
}

impl<F> TelemetryPublicationPort for RemoteTelemetryPublicationAdapter<F>
where F: FnMut(RemoteEffect) -> Result<RemoteObservation, RemotePortError>
{
    fn execute_telemetry_publication(&mut self, effect: RemoteEffect) -> Result<RemoteObservation, RemotePortError> {
        (self.execute)(effect)
    }
}
