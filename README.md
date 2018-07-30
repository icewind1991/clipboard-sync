# clipboard-sync

Synchronize clipboard between computers

## Usage

The project consists of 2 parts, a server that is responsible for passing messages between client
and the client which interact with the clipboard on the machine.

### Server

Simply run the server executable to start the server, by default the server listens on port 80,
this can be changed by setting the `PORT` environment variable.

A pre-built docker image with the server is available as `clipboardsync/server`

The server is currently only tested on linux, but should run on all platforms with
proper rust support.

### Client

When running the client, you need to specify the websocket address of the server and the session identifier to use,
all clients that share a session identifier will have their clipboards synced.

```bash
client ws://example.com my_session_key
```

The client is currently tested on linux(X11) and windows, but should also work fine on OSX and BSD.

## Building

clipboard-sync is build using rust and can be build using a simple

```bash
cargo build --release
```

which will put the server and client binary in `target/release`.

## Security

If multiple users have access to the machine running the clipboard-sync server it's important to take
care about security to prevent other users from snooping the clipboard contents.

- Put the sync server behind an SSL proxy
  
  This prevents the clipboard contents from being send over the network unencrypted.\
  Note that using ssl on windows is currently broken, the clipboard-sync should thus only
  be used in secure environments when windows support is required.

- Use a secure session key

  Everyone that knows the session key and has access to the sync server will be able
  to read the sync traffic. Picking a secure random key will prevent snooping of clipboard contents.