use crunch_remote::ExecutorPort;
use crunch_remote::RemoteEffect;
use crunch_remote::RemoteObservation;
use crunch_remote::RemotePortError;

pub struct LocalExecutorAdapter<F> {
    execute: F,
}

impl<F> LocalExecutorAdapter<F> {
    pub fn new(execute: F) -> Self {
        Self { execute }
    }
}

impl<F> ExecutorPort for LocalExecutorAdapter<F>
where F: FnMut(RemoteEffect) -> Result<RemoteObservation, RemotePortError>
{
    fn execute_build(&mut self, effect: RemoteEffect) -> Result<RemoteObservation, RemotePortError> {
        (self.execute)(effect)
    }
}

pub struct ExternalBatchExecutorAdapter<F> {
    execute: F,
}

impl<F> ExternalBatchExecutorAdapter<F> {
    pub fn new(execute: F) -> Self {
        Self { execute }
    }
}

impl<F> ExecutorPort for ExternalBatchExecutorAdapter<F>
where F: FnMut(RemoteEffect) -> Result<RemoteObservation, RemotePortError>
{
    fn execute_build(&mut self, effect: RemoteEffect) -> Result<RemoteObservation, RemotePortError> {
        (self.execute)(effect)
    }
}
