# mensa-rs

A terminal tool that shows today's menu for [Mensa am Schloss](https://www.stw-ma.de/essen-trinken/speiseplaene/mensa-am-schloss/) in Mannheim.

```
🍽  Mensa am Schloss — Mittwoch 18.03.
  Pasta.......... Spaghetti Bolognese (Rind, Ei, Weizen)
                  2,40 € / Portion
  Vegan.......... Linsensuppe mit Brot (Sellerie, Senf)
                  1,90 € / Portion
```

Meals containing milk (`Mi`) are highlighted in red.

## Installation

```bash
cargo install --path .
```

## Usage

```bash
mensa
```

Runs once and prints today's menu. The response is cached under `$XDG_CACHE_HOME/mensa/` (or `~/.cache/mensa/`) so repeated calls within the same day do not hit the network.
