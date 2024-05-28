# Dependabot Tracker

This a TUI application that tracks the total active dependabot alerts for a user. It uses the GitHub API to fetch the alerts and display them.

Note: You must create a [PAT](https://docs.github.com/en/authentication/keeping-your-account-and-data-secure/managing-your-personal-access-tokens#creating-a-fine-grained-personal-access-token) with the following minimum permissions:
- `Metadata: read-only`
- `Dependabot alerts: read-only`

## Running
To run the application, you must have Rust installed. You can install Rust by following the instructions at [rustup.rs](https://rustup.rs/). Once Rust is installed, just run `cargo run` in the root of the repository. You can rename `data/example_repositories.json` to `data/repositories.json` if you want to see example repositories.

## Environment Variables
There are three environment variables that must be set:
- `GH_USERNAME`: The GitHub username of the user to track (the PAT must for this user).
- `PAT`: The personal access token to use for authentication.
- `CARGO_PKG_NAME`: This will end up being the name of the logging file. It is recommended to set this to the name of the package (e.g., `dependabot-tracker`).

## Logging
By default, this application logs to `.data/dependabot-tracker.log` in the current working directory. On macOS and Linux, you can follow the log with `tail -f .data/dependabot-tracker.log`. There's probably something similar on Windows, but I don't know what it is.