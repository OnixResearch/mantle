use crunch_remote::RemoteEffect;
use crunch_remote::RemoteObservation;
use crunch_remote::RemotePortError;
use crunch_remote::StoreAdmissionPort;

pub struct RemoteStoreAdmissionAdapter<F> {
    execute: F,
}

impl<F> RemoteStoreAdmissionAdapter<F> {
    pub fn new(execute: F) -> Self {
        Self { execute }
    }
}

impl<F> StoreAdmissionPort for RemoteStoreAdmissionAdapter<F>
where F: FnMut(RemoteEffect) -> Result<RemoteObservation, RemotePortError>
{
    fn execute_admission(&mut self, effect: RemoteEffect) -> Result<RemoteObservation, RemotePortError> {
        (self.execute)(effect)
    }
}
