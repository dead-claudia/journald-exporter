# Contributing

This contains various notes about this crate, so it's a little easier to understand the code base.

- [Development](#development)
- [E2E testing](#e2e-testing)
- [Process isolation](#process-isolation)
- [Child server](#child-server)
- [Miri](#miri)

## Development

This assumes Rust 1.68.0 or later. You'll need a relatively recent toolchain, as it stays pretty close to the cutting edge.

You can find instructions on how to install that [here](https://www.rust-lang.org/tools/install).

## E2E testing

In order to run the E2E tests, you'll need three things installed: systemd, curl, and Bash. (Chances are, you have all three installed.)

First, before you run the E2E tests, run this command:

```sh
sudo ./scripts/e2e-setup.sh
```

It will set up all the stuff needed, including:

- The certificate and private key needed for testing HTTPS
- A `journald-exporter` system account, without a home directory
- A test API key

If you want to refresh it (ex: `/tmp` gets cleared), just re-run the script.

To run the tests, first build the binary:

```sh
# You *can* do a debug build, but it's better to test against the release build.
cargo build --release
```

Then, run one of the following:

```sh
# Test the plaintext HTTP endpoint (either one works)
sudo ./scripts/e2e.sh
sudo ./scripts/e2e.sh -t http

# Test the encrypted HTTPS endpoint
sudo ./scripts/e2e.sh -t https
```

There's a few other options you can pass along, too:

- `-p <PORT>`: Set the port to run at. Default is 8080.
- `-d <DURATION>`: Set the test duration to run in seconds. Default is 60. Note that anything below about 10 is pointless, since the API is invoked once every 5 seconds.
- `-b <BINARY_PATH>`: Set the path to the binary. CI is set up to use this to simplify its workflow.

## Process isolation

There are two main processes: the parent and child.

The parent process manages four things:

- Metrics state
- API key updates
- systemd journald reading
- Child process maintenance

The child process is focused solely on the server itself, exposing a single `/metrics` endpoint. It exposes a single `GET /metrics` endpoint over HTTP/1.1 that uses HTTP Basic Auth (username: `metrics`, password: an API key) for authorization.

> Why basic auth? It's just an API key and it's easy to integrate.

The parent process must be run as root so it can have full read access to the systemd journal. (This is also why Miri's used.) The child process runs in a dedicated `journald-exporter` user + group, isolated from the system, to limit the attack service strictly to the communication channel (which itself is *very* simplistic).

The parent and child can communicate in a limited fashion via an IPC channel. This channel is specially designed and specially read with a few things in mind:

1. Connection info is absent, so the parent cannot be impacted by anything that creates large client connection load or other similarly spammy requests.
2. The parent reads and processes metrics requests one request at a time, without buffering any response, to ensure that, in the unlikely event arbitrary code is executed on the client, it cannot overload the parent to the point it's unable to serve connections.

## Child server

The child server is laid out as a sort-of event driven server. The request flow at a high level works like this:

1. Request comes in, is queued for handling. The listener then loops back and waits for another request.
2. The request is removed from the queue, validated, and handled. If it's a metrics request with authorization, it's added to a list of requests pending metrics. If it's anything else, a response is just generated right then and there.
3. If the request is the first request to be added in the list, the parent is notified, and a metrics response is awaited.
4. Once the metrics response body is received, it's broadcasted to all requests in the list.
5. If any error occurs while waiting, all requests instead have a 503 Service Unavailable response broadcasted to them, and the error itself is logged.

## Miri

[Miri](https://github.com/rust-lang/miri) is used both locally and in CI for two reasons:

1. Ensure memory accesses in unsafe code are still as safe as pragmatically possible.
2. Help track down concurrency issues - both the child and parent are multi-threaded, so this is pretty important.

Sometimes, Miri will fail, or it might just run extremely slow.

- For single tests, just disable it with a `#[cfg_attr(miri, ignore)]` and leave a comment why.
- If all the tests in a module are impacted, just disable the whole `tests` module.

There's two main reasons you'll need to disable tests:

1. Unavoidable FFI calls and filesystem calls. Miri doesn't support that in isolated mode (what this uses) and generally never will with very few exceptions.
2. It's extremely slow. Miri isn't fast in any sense of the word, and a full run can take around 20-30 minutes to complete depending on your machine. **If you plan to disable a test for this reason, make sure a smaller test also exists so that part of the code is still covered by Miri.**

There are ways to mitigate both of those:

- If it's not "just" a simple FFI wrapper, you can likely shim it.
- Instead of using libc helpers, one can just implement the algorithm in Rust. This isn't common for FFI-related code, but built-in Rust methods are typically used in place of libc methods.
- Constants and static variables can be used to hack out a *lot* of potential slowness, at only a modest increase in compile time. They are heavily limited, but strategic use of arrays can go a long way.
- If you're doing a lot of `Vec` work and it's turned out to be really slow in Miri, consider using `Vec::with_capacity`. The resizing operation is pretty slow, and if you're doing it in a loop, you'll probably save a lot of time.

Also, there's one thing to call out: logs are *not* captured in Miri. They are explicitly ignored, and help is very much appreciated in figuring out why logs, even when using the internal test-specific capture logger, aren't handled by Miri.
