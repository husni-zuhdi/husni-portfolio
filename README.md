# Husni Portfolio
My Portfolio webiste

## Tools I use in this repo
* [Rust Programming Language](https://www.rust-lang.org/)
* [Axum](https://github.com/tokio-rs/axum/tree/main)
* [Askama](https://github.com/djc/askama)
* [Markdown-rs](https://github.com/wooorm/markdown-rs)
* [Octocrab](https://github.com/XAMPPRocky/octocrab)
* [Taskfile](https://taskfile.dev/)
* [TailwindCSS](https://tailwindcss.com/)

## How to develop

Please follow conventional commit style.

```
# Run hot reloading
task run

# Run unit test
task test

# Build docker image
task docker-build
```

## How to use
1. Build the docker image.
2. Push to your docker registry.
3. Set your container service (cloud-run, k8s, fargate, linode, docker swarm, etc) to use this image and set the environment varaibles in the [env.example](./env.example) file.
