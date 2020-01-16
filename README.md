# Terra Mach

Terra Mach is a frontend system to build performant graphical interfaces for desktop, embedded, and mobile systems.

The project is in active development. Most of the APIs are stable, though some breaking changes are still possible.

This project is highly inspired by [Flutter](https://flutter.dev). Terra Mach is written in a systems programming language [Rust](https://www.rust-lang.org). 
It leverages graphics library [Skia](https://skia.org) to enable high performant 2D graphics.

# How to Use

Terra Mac create is located in [terramach](/terramach) folder. To use it checkout Terra Mach to your workspace and link 
it locally by supplying a path.

```shell script
git clone https://github.com/lykhonis/terramach.git
cd MyProject
```

Add dependency in `Cargo.toml`.

```toml
[dependencies.terramach]
path = "../terramach/terramach"
```

## Examples

Terra Mach comes with some prebuilt examples of what can be built with it.

### Dashboard

![Dashboard Preview](/docs/assets/dashboard.gif)

A dashboard sample app inspired by [Dark Version](https://dribbble.com/shots/3530048-Dark-Version) design. 
The dashboard integrates [Mapbox](https://www.mapbox.com) to access maps. The integration module is located in 
[third-party crate](/third-party/mapbox).

Try example by running in command line:
```shell script
cd examples/dashboard
cargo run --release
```

This may take awhile for initial build, so don't hesitate to grab some :coffee:.

In order to access maps, you would need to supply Mapbox access token in [Settings.toml](/examples/dashboard/Settings.toml).
Register and get Mapbox access token by signing up [here](https://account.mapbox.com/auth/signup).

```shell script
cd examples/dashboard
touch Settings.toml
```
and insert following content:
```toml
[mapbox]
access-token = "ACCESS_TOKEN_GOES_HERE"
cache-path = "/tmp/mapbox.cache.db"
```

## Supported Platforms

| Platform | Status      |
| -------- | ----------- |
| Mac OS   | Supported   |
| Android  | Planned     |
| Linux    | Planned     |
| Windows  | Planned     |
| Web      | Considered  |
| iOS      | Not planned |

# License

This software is available publicly via GPLv3 license which can be found [here](/LICENSE). 
For any other request please [contact me](mailto:vladimirlichonos@gmail.com).
