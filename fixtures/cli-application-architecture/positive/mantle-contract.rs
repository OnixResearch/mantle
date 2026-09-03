struct ApplicationCommand {
    operation: &'static str,
}

struct ApplicationObservation {
    effect_id: &'static str,
}

trait BuildOperationPort {
    fn execute(&mut self, command: ApplicationCommand) -> ApplicationObservation;
}

fn dispatch(port: &mut impl BuildOperationPort, command: ApplicationCommand) -> ApplicationObservation {
    port.execute(command)
}
