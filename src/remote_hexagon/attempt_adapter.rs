use crunch_remote::AttemptPersistencePort;
use crunch_remote::RemoteEffect;
use crunch_remote::RemoteObservation;
use crunch_remote::RemotePortError;

pub struct RemoteAttemptPersistenceAdapter<F> {
    execute: F,
}

impl<F> RemoteAttemptPersistenceAdapter<F> {
    pub fn new(execute: F) -> Self {
        Self { execute }
    }
}

impl<F> AttemptPersistencePort for RemoteAttemptPersistenceAdapter<F>
where F: FnMut(RemoteEffect) -> Result<RemoteObservation, RemotePortError>
{
    fn execute_attempt(&mut self, effect: RemoteEffect) -> Result<RemoteObservation, RemotePortError> {
        (self.execute)(effect)
    }
}
