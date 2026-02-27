# Quick Start

## 1. Install the Heroku CLI

```bash
brew tap heroku/brew && brew install heroku
```

Or see [heroku.com/cli](https://devcenter.heroku.com/articles/heroku-cli) for other platforms.

## 2. Log in to Heroku

```bash
heroku login
```

## 3. Run the app

**From a release build:**

```bash
./target/release/logs-parser-desktop
```

**From source:**

```bash
cargo run --release --bin logs-parser-desktop
```

## 4. Connect to an app

1. The app fetches your Heroku apps on startup
2. Select an app from the dropdown
3. Click **Connect**
4. Logs stream in real time

## 5. Filter logs

Type a filter expression and press **Enter**:

- `error` — text search
- `dyno:web.1` — specific dyno
- `level:error` — log level
- `/pattern/` — regex

Press **C** to clear all filters.
