# taskbar-weather 0.1.0
Some sort of weather/temperature showing layer thingy.

# Requires
- Internet.

# Compiling
Issue either:
- `cargo build` or
- `cargo build --release`

depending on whether you want debug clutter or not. For pure headless
CLI-mode, add `--features "headless"`, e.g.:
- `cargo build --release --features "headless"`

# Running
**taskbar-weather** works as both UI-widget/overlay
and as a command-line tool. UI, however, is at the moment Windows-only
feature — I'll get to other system(s) UI's later.
## Command Line
Issue either:
- `taskbar-weather.exe --help` or
- `./taskbar-weather --help`

(at wherever you or cargo did put them). Should bring up something like this
(as per time of writing):
```text
Taskbar-Weather — a tool to fetch your local weather. See LICENSE.

Usage: taskbar-weather.exe [OPTIONS]

Options:
      --headless           If given, doesn't use UI and logs to console instead
  -o, --oneshot            If given, checkes weather once, logs it to console, and quits right after
      --city <CITY>        If given, overrides configured city setting(s)
      --country <COUNTRY>  If given, overrides configured country setting(s)
  -h, --help               Print help
  -V, --version            Print version
```

Note that if built (or ran) with `--feature "headless"`, the app will
naturally enforce `--headless` running (if no `--oneshot` is given, that is).

# Configuration
By default, when first ran, the app creates an example `config.toml`
file to OS-specific configuration directory.

### Windows
Location of config file:
- `C:\Users\USERNAME\AppData\Roaming\msukanen\TaskbarWeather\config\config.toml`

## Contents of `config.toml`
```toml
city = "YOUR CITY"
country = "CC"
```
…where `CC` stands for "country code", e.g. `FI`, `GB`, `US`, etc.

# Backstory
Yeah, M$ decided to break their outdoors temperature showing thingydoodah (probably with Win11 24H2 update…),
so I made my own.

## Fighting with Windows
Not for the faint of heart, just saying.

The code to hide this weather widget from alt-tab and taskbar
(so that it neatly sits on the screen as a quiet, non-intrusuve and
non-interactable "display" with temperature data) is a boatload
of dark arts and bitter hate toward M$ devs. More complicated mess
of spaghetti than anything else in the app.

The hybrid UI/console compilation is somewhat messy business with
windoze's subsystems. Code compiled with UI enabled (default) has
a guard against `--headless` running as the headless mode would
run as a detached "ghost" which you can't kill without Task Manager
(the app is in reality truly GUI one even though it can print to
console).
