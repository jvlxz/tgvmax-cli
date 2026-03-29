# tgvmax-cli

Find available TGVmax (Max Jeune) 0€ trains on SNCF.

## Installation

```bash
# Shell installer (macOS / Linux)
curl --proto '=https' --tlsv1.2 -LsSf https://github.com/jvlxz/tgvmax-cli/releases/latest/download/tgvmax-cli-installer.sh | sh

# Or from crates.io
cargo install tgvmax-cli
```

## Usage

### Search for stations

```bash
$ tgvmax station list -s Paris
```

```
+--------------------------+
| Station Name             |
+==========================+
| PARIS (intramuros)       |
+--------------------------+
| PARIS MONTPARNASSE       |
+--------------------------+
| MASSY TGV / PARIS (RER) |
+--------------------------+
```

### Find available trains

```bash
$ tgvmax train search -f Paris -t Lyon -d 01/04/2026
```

```
+---------+-----------+---------+--------------------+--------------------+
| Train # | Departure | Arrival | From               | To                 |
+=========+===========+=========+====================+====================+
| 6607    | 09:00     | 10:56   | PARIS (intramuros) | LYON (intramuros)  |
+---------+-----------+---------+--------------------+--------------------+
| 6609    | 11:00     | 12:56   | PARIS (intramuros) | LYON (intramuros)  |
+---------+-----------+---------+--------------------+--------------------+
| 6641    | 17:00     | 18:56   | PARIS (intramuros) | LYON (intramuros)  |
+---------+-----------+---------+--------------------+--------------------+
```

Omit `--date` to search today. Dates use `DD/MM/YYYY` format.

### JSON output

Add `--json` to any command:

```bash
$ tgvmax train search -f Paris -t Lyon -d 01/04/2026 --json
```

```json
[
  {
    "train_number": "6607",
    "departure": "09:00",
    "arrival": "10:56",
    "origin": "PARIS (intramuros)",
    "destination": "LYON (intramuros)"
  },
  {
    "train_number": "6609",
    "departure": "11:00",
    "arrival": "12:56",
    "origin": "PARIS (intramuros)",
    "destination": "LYON (intramuros)"
  }
]
```

### Refresh station cache

Station data is cached locally for 24 hours. Force a refresh with `--refresh`:

```bash
$ tgvmax station list -s Lyon --refresh
```

## License

MIT
