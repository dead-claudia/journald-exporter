use crate::prelude::*;

use super::state::ParentIpcState;
use super::types::*;
use crate::parent::key_watcher::write_current_key_set;

#[must_use]
fn read_request(
    s: &'static ParentIpcState<impl ParentIpcMethods>,
    buf: &[u8],
) -> ipc::child::DecoderRequest {
    let mut decoder = s.decoder().lock();
    decoder.read_bytes(buf);
    decoder.take_request()
}

pub fn write_to_child_input(s: &'static ParentIpcState<impl ParentIpcMethods>, buf: &[u8]) -> bool {
    let mut guard = s.child_input();

    let Some(stdin) = &mut *guard else {
        return false;
    };

    let result = try_send_msg(s.done_notify(), stdin, buf);

    if !result {
        *guard = None;
    }

    result
}

fn try_handle_metrics_request(
    s: &'static ParentIpcState<impl ParentIpcMethods>,
) -> io::Result<Vec<u8>> {
    let table = s.methods().get_user_group_table()?;
    if let Some(snapshot) = s.state().snapshot() {
        let environment = &s.dynamic().prom_environment;
        if let Some(result) = render_openapi_metrics(environment, &snapshot, &table) {
            return Ok(result);
        }
    };

    Err(Error::from_raw_os_error(libc::ENOMEM))
}

#[must_use]
fn handle_metrics_request(s: &'static ParentIpcState<impl ParentIpcMethods>) -> bool {
    let result = try_handle_metrics_request(s).unwrap_or_else(|e| {
        log::error!("{}", normalize_errno(e, None));
        Vec::new()
    });
    write_to_child_input(s, &result)
}

pub fn ipc_message_loop<M: ParentIpcMethods>(
    mut child_output: M::ChildOutput,
    s: &'static ParentIpcState<M>,
) -> io::Result<()> {
    // 4 bytes is far more than enough to read client IPC messages efficiently, since they're
    // all just one byte and they're always batched into one go. In practice, there's really
    // only going to be 1-2 bytes to read total.
    let mut read_buf = [0_u8; 4];

    while let Some(buf) = try_read(&mut child_output, s.done_notify(), &mut read_buf)? {
        let request = read_request(s, buf);

        // This is done sequentially, as it in practice is only hit up to about once a minute.
        s.state()
            .add_metrics_requests(request.tracked_metrics_requests());

        if request.keys_requested() && !write_current_key_set(s) {
            break;
        }

        if request.metrics_requested() && !handle_metrics_request(s) {
            break;
        }
    }

    Ok(())
}
