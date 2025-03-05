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

You can get a quick health check without starting the server using the following command:

```bash
$ healthmonitor check
```

This does a limited number of fast checks to get a quick status of the environment. It is intended to be used very
early in the deployment process to ensure the environment is stable enough to start the monitored application.

### Setting and getting the deployment phase

When the server is started, the application will be by default in "Deploying" state. During this phase the application
can do deployment specific tasks, such as setting up the database schema, importing configuration, etc. After the
deployment is complete the application can be set to "Online" state.

During the "Deploying" phase the health status is always set to "healthy". This is to ensure that the cloud orchestrator
will not terminate the instance before the deployment is complete.

To set the deployment phase to "Deploying" or "Online", use the following command:

```bash
$ healthmonitor phase set <phase>
```

Example:

```bash
$ healthmonitor phase set online
```

### Scripting

A typical deployment script of a monitored application will look like this:

```bash
#!/usr/bin/env bash

# Do a quick sanity check of the environment before starting the deployment.
# This will fail with an error code if the environment is not healthy.
healthmonitor check

# Start the health monitor in the background.
healthmonitor server start &

# Switch the health monitor to deployment mode. This will ensure the instance will not be marked unhealthy until the
# deployment is complete.
healthmonitor phase set deploying

# Perform the deployment of the application.
drush deploy

# When deployment is complete, switch the health monitor from deployment mode to online mode.
healthmonitor phase set online
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

Get the current health status of the application: http://127.0.0.1:8080/status - it will return 200 OK if the
application is healthy, and 503 Service Unavailable if the application is unhealthy.

The available REST endpoints are documented in [server.http](https://github.com/pfrenssen/healthmonitor/blob/master/server.http).

## Built-in checks

### File check

The file check plugin checks for the existence of a list of files on the server instance. If any of the files is missing
or empty, the application will be marked as unhealthy.

It can be configured using the `HEALTHMONITOR_FILECHECK_*` environment variables.

### URL check

The URL check plugin checks if a list of URLs are reachable and return a 200 OK status code. If any of the URLs are
unreachable or return a non-200 status code, the application will be marked as unhealthy.

Configuration is done using the `HEALTHMONITOR_URLCHECK_*` environment variables.
