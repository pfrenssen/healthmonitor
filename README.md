# HealthMonitor

HealthMonitor is a simple program designed to monitor the health status of a web based application. It provides both a
command-line interface and a RESTful API to query and set the health status. It is intended to be integrated with
internal cloud instance monitoring systems or load balancers to ensure the reliability and availability of your service.

## Installation

This is a Rust application which can be compiled using Cargo:

```bash
$ cargo build --release
```

The application is configured using environment variables. The default values are documented in `.env.example`. Set them
in your instances using your cloud provider's environment variable configuration. You could also create a `.env` file in
the same directory as the binary to set the environment variables.


## CLI

### Starting the server

The server needs to be started before it can be used to monitor the health status of the application. When the server is
started it will keep running in the foreground until it receives a termination signal (e.g. `Ctrl+C`).

```bash
$ healthmonitor server start
```

To check if the server is running, use the following command:

```bash
$ healthmonitor server status
```


### Setting and getting the health status

When the server is started, the health status is set to `healthy` by default. You can retrieve the current health status
using the following command:

```bash
$ healthmonitor status get
```

To set the health status to `healthy` or `unhealthy`, use the following command:

```
healthmonitor status set <health_state> [--message <MESSAGE>]

• <health_state>: The new health status (healthy or unhealthy).
• --message: (Optional) A custom message describing the status change.
```

Example:

```bash
$ healthmonitor status set unhealthy --message "Cannot connect to database."
```


## REST endpoint

A REST endpoint is exposed by the application to interact with the health status through HTTP requests.

**WARNING**: This endpoint is only intended for internal use inside the private cloud network. It is intended to be
called by local services such as load balancers or monitoring systems. There is absolutely no security or access control
implemented. Do not expose this endpoint to the public internet, or unauthorized users may be able to change the health
state of your application and cause a denial of service.

The port on which the server listens is configurable using the `HEALTHMONITOR_SERVER_PORT` environment variable. The
default port is `8080`.

Check if the server is running: http://127.0.0.1:8080/info

Get the current health status of the application: http://127.0.0.1:8080/status

The available REST endpoints are documented in [server.http](https://github.com/pfrenssen/healthmonitor/blob/master/server.http).