# Husni Portfolio
My Portfolio webiste

## Tools I use in this repo
* [Rust Programming Language](https://www.rust-lang.org/)
* [Axum](https://github.com/tokio-rs/axum/tree/main)
* [Askama](https://github.com/djc/askama)
* [Markdown-rs](https://github.com/wooorm/markdown-rs)
* [Taskfile](https://taskfile.dev/)
* [TailwindCSS](https://tailwindcss.com/)
* [Turso](https://turso.tech/)
* [Google Cloud Storage](https://cloud.google.com/storage)

## How to develop

Please follow [Conventional Commit Style](https://www.conventionalcommits.org/en/v1.0.0/).

```
# Run hot reloading
task run

# Run unit test
task test

# Linting
task lint

# Audit and Security scanning
task audit

# Code Coverage
task coverage

# Build docker image
task docker-build
```

## How to use
1. Build the docker image by push a new tag on this repo.
2. Push to your docker registry (or use this push and build pipeline).
3. (Optional) If you use `turso`, setup your sqlite database.
4. (Optional) If you store secrets in `Google Cloud Storage`, setup your bucket and Google Cloud Platform credentials [[example for local device]](https://docs.cloud.google.com/docs/authentication/set-up-adc-local-dev-environment).
4. Set your container service (cloud-run, k8s, fargate, linode, docker swarm, etc) to use this image and set the environment varaibles in the [env.example](./env.example) file.
