use crunch_remote::RemoteEffect;
use crunch_remote::RemoteObservation;
use crunch_remote::RemotePortError;
use crunch_remote::TransportPort;

pub struct StdioTransportAdapter<F> {
    execute: F,
}

impl<F> StdioTransportAdapter<F> {
    pub fn new(execute: F) -> Self {
        Self { execute }
    }
}

impl<F> TransportPort for StdioTransportAdapter<F>
where F: FnMut(RemoteEffect) -> Result<RemoteObservation, RemotePortError>
{
    fn execute_transport(&mut self, effect: RemoteEffect) -> Result<RemoteObservation, RemotePortError> {
        (self.execute)(effect)
    }
}

pub struct SshTransportAdapter<F> {
    execute: F,
}

impl<F> SshTransportAdapter<F> {
    pub fn new(execute: F) -> Self {
        Self { execute }
    }
}

impl<F> TransportPort for SshTransportAdapter<F>
where F: FnMut(RemoteEffect) -> Result<RemoteObservation, RemotePortError>
{
    fn execute_transport(&mut self, effect: RemoteEffect) -> Result<RemoteObservation, RemotePortError> {
        (self.execute)(effect)
    }
}
