# taskbar-weather 0.1.0
Some sort of weather/temperature showing layer thingy.

# Requires
- Internet.

# Configuration
By default, when first ran, the app creates an example `config.toml`
file to OS-specific configuration directory. This isn't yet truly
cross-compilable (it does compile about anywhere, but UI stuff is
at the moment tuned only for/with Win11).

### Windows
Location of config file:
- `C:\Users\USERNAME\AppData\Roaming\msukanen\TaskbarWeather\config.toml`

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
