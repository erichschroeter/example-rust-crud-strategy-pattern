# example-rust-crud-strategy-pattern
A minimal example of a CRUD web app that stores data to multiple backend stores.

# Design
### Common
The `common` project contains code common between the `frontend` and `backend` projects.

### Backend
The `backend` handles storing the CRUD storage.

### Frontend
The `frontend` handles the user interface to the `backend`.
[Tera](https://keats.github.io/tera/) is used for server-side templating web pages.

# Building

1. Install dev dependencies
```bash
sudo apt install build-essential
```
1. Build
```bash
cargo build
```
