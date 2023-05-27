# shremdup

![GitHub release (latest by date)](https://img.shields.io/github/v/release/DiscreteTom/shremdup?style=flat-square)
[![GitHub Workflow Status](https://img.shields.io/github/actions/workflow/status/DiscreteTom/shremdup/build.yml?style=flat-square)](https://github.com/DiscreteTom/shremdup/actions/workflows/build.yml)
![license](https://img.shields.io/github/license/DiscreteTom/shremdup?style=flat-square)

Call Windows Desktop Duplication API through gRPC and shared-memory.

## Download

From [releases](https://github.com/DiscreteTom/shremdup/releases).

## Usage

```sh
shremdup.exe <port>
```

> **Note**: if you use shared memory starts with `Global\`, you need to run `shremdup.exe` as administrator.

## Protocol

See [shremdup.proto](https://github.com/DiscreteTom/shremdup/blob/main/proto/shremdup.proto) for the gRPC definition.

## Build

First, install `protoc` v23.2 following https://github.com/hyperium/tonic#windows.

Then, run:

```bash
cargo build --release
```

## Related

- [rusty-duplication](https://github.com/DiscreteTom/rusty-duplication) - the underlying library that calls Windows Desktop Duplication API and manage shared-memory.
- [HyperDesktopDuplication](https://github.com/DiscreteTom/HyperDesktopDuplication) - a Unity3D library to render Windows desktops in Unity3D games.

## [CHANGELOG](https://github.com/DiscreteTom/shremdup/blob/main/CHANGELOG.md)
