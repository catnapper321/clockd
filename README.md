# clockd
A daemon that keeps time and manages an alarm queue. This is a demo application
featuring:

- Async with custom futures
- Interprocess communication and fd passing via unix sockets
- Command line parsing with the clap crate
- Child process management
