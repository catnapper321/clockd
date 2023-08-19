use std::{
    fs::{self, File},
    io::{IoSliceMut, Write},
    os::unix::{
        io::{self, FromRawFd, OwnedFd, RawFd},
        net::{AncillaryData, SocketAncillary, UnixDatagram},
    },
    sync::Mutex,
    path::{Path, PathBuf},
};
use tracing::{debug, error};
use crate::{broadcast, parker::Nudger};

static FD_LIST: Mutex<Vec<Option<File>>> = Mutex::new(Vec::new());

fn prep_fd_socket(path: impl AsRef<Path>) -> std::io::Result<()> {
    let path = path.as_ref();
    if PathBuf::from(path).exists() {
        fs::remove_file(path)?;
    }
    Ok(())
}

fn recv_stdout(sock: &UnixDatagram) -> std::io::Result<File> {
    let mut buf1 = [1; 1];
    let mut bufs = &mut [IoSliceMut::new(&mut buf1)][..];
    let mut fds = [0; 8];
    let mut ancillary_buffer = [0; 128];
    let mut ancillary = SocketAncillary::new(&mut ancillary_buffer[..]);
    _ = sock.recv_vectored_with_ancillary_from(bufs, &mut ancillary)?;
    for ancillary_result in ancillary.messages() {
        if let Ok(AncillaryData::ScmRights(mut scm_rights)) = ancillary_result {
            let fd = scm_rights.next();
            if let Some(fd) = fd {
                let f = unsafe { File::from_raw_fd(fd) };
                // let fd = unsafe { OwnedFd::from_raw_fd(fd) };
                return Ok(f);
            } else {
                error!("ancillary data did not have an fd");
            }
        } else {
            error!("message had no ancillary data");
        }
    }
    Err(std::io::ErrorKind::Other.into())
}

/// call with spawn_blocking. relays received fds via channel.
// not async because async_std net stuff does not support fd passing
pub fn start_fd_socket(path: impl AsRef<Path>, mut nudger: Nudger) {
    prep_fd_socket(&path);
    let socket = UnixDatagram::bind(path).expect("Could not bind path for unix socket: {path:?}");
    loop {
        let fd = recv_stdout(&socket);
        if let Ok(fd) = fd {
            debug!("Received fd {fd:?}");
            {
                let mut fds = FD_LIST.lock().unwrap();
                fds.retain(|f| f.is_some());
                fds.push(Some(fd));
            }
            // send message back to the main loop
            nudger.nudge();
        } else {
            error!("Bad message on fd socket");
        }
    }
    unreachable!()
}

use serde::Serialize;
pub fn print_json_to_fds(ser: &impl Serialize) -> serde_json::error::Result<()> {
    let json_string = serde_json::to_string(ser)?;
    let mut fds = FD_LIST.lock().unwrap();
    for x in fds.iter_mut() {
        if let Some(f) = x {
            if writeln!(f, "{}", json_string).is_err() {
                error!("fd {f:?} closed");
                *x = None;
            }
        }
    }
    Ok(()) 
}
