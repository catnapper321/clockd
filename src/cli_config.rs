use crate::*;

const FD_SOCKET_NAME: &str = "clockd.fd";
const CMD_SOCKET_NAME: &str = "clockd.cmd";

#[derive(clap::Parser, Debug)]
pub struct Config {
    /// port for webserver to listen on
    #[clap(short = 'p', long = "port", default_value = "3001")]
    pub port: u16,
    /// path to the unix socket for application commands
    #[clap(short = 's', long = "socket")]
    pub cmd_socket: Option<PathBuf>,
    /// path to the unix socket for fd passing (stdout shim)
    #[clap(short = 'f', long = "fdsocket")]
    pub fd_socket: Option<PathBuf>,
    /// logging level
    #[clap(short = 'v', action = clap::ArgAction::Count)]
    pub verbosity: u8,
}

/// mutates the passed reference into a path for the socket.
/// if Some(x), does nothing. If None, constructs a default socket path.
fn make_socket_path(config_path: &mut Option<PathBuf>, default_name: &str) -> Anything<()> {
    if config_path.is_some() {
        return Ok(());
    }
    // try to construct a path from $XDG_RUNTIME_DIR
    if let Ok(d) = std::env::var("XDG_RUNTIME_DIR") {
        let mut socket_path = PathBuf::new();
        socket_path.push(d);
        socket_path.push(default_name);
        debug!("Using default socket path: {default_name}");
        *config_path = Some(socket_path);
        Ok(())
    } else {
        let msg = format!("socket path must be specified for {default_name}");
        error!(msg);
        Err(msg.into())
    }
}

pub fn get_config() -> Anything<Config> {
    let mut c: Config = clap::Parser::parse();
    make_socket_path(&mut c.fd_socket, "clockd.fd")?;
    make_socket_path(&mut c.cmd_socket, "clockd.cmd")?;
    Ok(c)
}
