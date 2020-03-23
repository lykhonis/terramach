# Terra Mach

Terra Mach is a frontend system to build performant graphical interfaces for desktop, embedded, and mobile systems.
It aims to make it easy to create a branded user experience. It does not obey a platform's look and feel,
though it provides tools to create similar experiences.

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

## Build a Widget

TerraMach GUI is a composition of widgets and/or direct painting. For example, a Decoration widget
paints background color but also manages its child widget.

A simple example of a counter app. On a tap, the counter is increased and UI is updated to reflect the change.

1. Define a widget and its state (state is optional)
```rust
#[derive(Default, Clone, PartialEq, PartialWidget)]
struct Counter {}

#[derive(Default)]
struct CounterState {
    counter: usize,
}
```

2. Implement a counter widget
```rust
impl Widget for Counter {
    // prepare state for the counter
    fn mount(&self, context: &mut WidgetContext, mount: &mut MountContext) {
        context.set_state(CounterState::default());
    }
    
    // build a counter widget with tap gesture and white background
    fn build(&self, context: &mut WidgetContext, build: &mut BuildContext) {
        let state = context.state::<CounterState>().unwrap();
        build.add_child(
            Gesture::new(
                0,
                build.event_emitter(),
                TapGesture::default(),
                None,
                Decoration::new(
                    Color::WHITE,
                    None,
                    Align::new(
                        Alignment::center(),
                        Text::new_text(format!("Counter {}", state.counter).as_str()),
                    ),
                ),
            ),
        );
    }
    
    // handle a single tap
    fn event(&self, context: &mut WidgetContext, event: &mut EventContext) {
        if let Event::Tap(_) = event.get() {
            let state = context.state_mut::<CounterState>().unwrap();
            state.counter += 1;
            event.mark_need_build();
        }
    }
}
```

3. Start the app
```rust
fn main() {
    App::new((1020, 640))
        .with_title("Counter")
        .run(Counter::default());
}
```

## Supported Platforms

In order:

| Platform | Status      |
| -------- | ----------- |
| Mac OS   | Supported   |
| Android  | Planned     |
| Linux    | Planned     |
| Windows  | Planned     |
| iOS      | Planned     |
| Web      | Considered  |

# License

This software is available publicly via GPLv3 license which can be found [here](/LICENSE). 
For any other request please [contact me](mailto:vladimirlichonos@gmail.com).
