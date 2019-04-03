# PubNub NATs Bridge

Bring push notifications and streaming events to Mobile and Web clients
based on a NATs channels.
Easy drop-in sidecar.
Dashboard management page included.

The application is written in Rust.
The application will listen on NATS channels specified by runtime
and deploytime configuration.
Built-in security model allows encyrpted data in motion using 2048bit TLS.

## Building the Docker Container

Production runtime image size is 6MB.

```shell
docker build --cpuset-cpus 3 -t nats-bridge .
```

## Deploying Sidecar in Kubernetes (K8s)

...
Add Secrets to secret store.
Add YAML sidecar

## Deploying Docker Compose

`docker run -p 4222:4222 nats -p 4222`

...

## Reference Links

https://hub.docker.com/_/nats
