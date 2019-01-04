# Heroku Log Drain for forwarding logs to DataDog

**DISCLAIMER**: This is my first Rust project and its primary purpose is for me
to play with Rust a bit, not to build a production-ready piece of software. That
being said, if you want this to exist, feel free to open pull requests, whether
completing missing functionality or providing helpful refactoring that will
teach me more about writing idiomatic Rust.

## Overview

While DataDog accepts logs over syslog and Heroku, notionally, supports draining
over syslog, they can't currently be configured to work directly together
because each party is specific about how authentication and application
identification works.

This project is designed to run as a Heroku application, accept Heroku Logplex
logs over HTTPS and forward them to Datadog.

Eventually, it could potentially be a generic way to forward Heroku logs to
arbitrary destinations that aren't natively compatible with Heroku Drains.

## License

[MIT licensed](https://tldrlegal.com/license/mit-license). See `LICENSE` file
license text.
