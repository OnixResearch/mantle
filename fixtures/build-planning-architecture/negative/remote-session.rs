fn open(command: crunch_remote_core::RemoteCommand) {
    let _ = crunch_remote_core::start_remote_session(command);
}
