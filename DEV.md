Getting Started

GlyphOS aims to keep things simple. To get started, simply clone the repository, and run the project.


git clone https://github.com/bartekus/GlyphOS
cd GlyphOS
cargo run

# Workspaces

GlyphOS makes use of Cargo Workspaces to manage and structure its codebase.
As such, the main portion of the CLI functionality is inside src folder.
CLI functionality is to start the program, the configuration, logging, and then run the arguments parser.
It's not supposed to have any other functionality.

It is recommended that these are added through Workspaces.
GlyphOS comes with 3 workspaces.

cli : Configure and start Clap.
core : This is where GlyphOS business logic ought to be.
utils : Various utilities like logging, configuration, errors, etc...

These different workspaces are libraries.
It's preferrable that new workspaces can compile on their own.
This makes the code modular and unit testing much simpler.

# Clap

GlyphOS uses clap to parse Command Line arguments.
clap is simple, efficient and widely used by the community.
clap configuration code can be found in cli/src/lib.rs.

# Errors

GlyphOS tries to follow the best practices to handle errors.
For this, it is useful that GlyphOS program uses Rust's std::error::Error trait.
All the errors should implement this trait.
To be able to propagate errors using the ? operator, it's possible to wrap your custom error type with a Box type.
This has the downside of dynamically allocating memory for errors in the heap.
A better approach is to convert errors at compile time; which GlyphOS has a few examples of.

The approach taken is to define proper Result and Error types.
This has the advantage of easy upgrades and library changes.
For example, if you want to switch from the thiserror crate,
you have a central and single place to do this change, and the rest of your code is unchanged.

Simulating an error

To test that our error reporting is functioning correctly, GlyphOS has a special subcommand to intentionally trigger an error.

cargo run error

GlyphOS uses the better-panic crate to print and prettify backtraces.
This library is only enabled when debugging and disabled for release builds.

Backtracing for the std::Error is only available in nightly.
For this reason, the stable GlyphOS still uses the Failure crate.
If you want to use std::Error with Backtrace in nightly, use the newError branch.

# Configuration

GlyphOS manages a configuration struct to mutate, store and retrieve a configuration state.
Currently, a configuration file could be passed using the -c flag, and its content will be merged.
Environment variables starting with APP will also be merged.
It's recommended to change the APP prefix.

Under the hood, GlyphOS uses the config-rs crate.
However, it exposes an AppConfig struct that can be use to store and retrieve configuration.
The relevant code can be found in utils/src/app_config.rs.
It's recommended to use this struct instead of config-rs directly,
as this dependency might get changed in the future.

The configuration struct is initialized with a default configuration file at program start.
This file can be found in src/resources/default_config.toml.
You only need to initialize the configuration once.
It's stored in a static RwLock making it globally accessible and also thread-safe.


    let config_contents = include_str!("resources/default_config.toml");
    AppConfig::init(Some(config_contents))?;

You can merge additional settings to your configuration struct, but also retrieve or change single settings.


    // Merge config
    AppConfig::merge_config(cli_matches.value_of("config"))?;
    // Get config value
    AppConfig::get::("debug").unwrap();
    // Set config value
    AppConfig::set("database.url", "new url").unwrap();
# Logging

The standard interface for logging in Rust is the log crate.
It defines a set of macros for logging like info! and error!.
While GlyphOS uses slog and has two drains for logs: syslog and terminal;
it is fully compatible with the log facade.

Logging in GlyphOS works out of the box.
In any of the sub-crates in your workspace, you can add the log crate as a dependency and use its macros.
These logs will be forwarded to slog drains.
If these drains are not available or fail to initialize, the logs will be discarded.

To see logging in action, you can run the error command.
It should log an error message to the terminal and syslog.
Before the program fails, you should see this line.
(This should also be logged to syslog).

July 16 01:19:42.014 INFO We are simulating an error, who: GlyphOS
While this is good enough, you might want to customize your logging drains.
Any logs that you write, are passed to all of these drains. slog supports several drain options.
The file utils/src/logger.rs contains these implementations.

# Testing

GlyphOS comes with a few tests for the commandline program and the config struct.
You can run all the tests by running.

cargo test --all
It is recommended to put integration tests for the command line in the root of your program,
and write tests for each sub-crate in your workspace individually.

GlyphOS also has integrations to run tests in Github actions, and also a code coverage integration with codecov.

# CI/CD

GlyphOS is not optimized for any particular use-case;
and should, preferably, run in multiple operating systems under different conditions.
Unfortunately, it's currently compiled only against Linux.
macOS and Windows are high in the list of the OSes to be supported in the next releases.

Currently, GlyphOS compiles under rust-musl-builder.
rust-musl-builder is a docker image to compile your rust application into a static Rust binary.
It is then tested under Alpine Linux to make sure your program is working inside the lightest of containers.

While compiling locally is faster for development, this doesn't give many guarantees about your program running under different conditions.
This is less important if you are deploying your program to a predictable environment (ie: a server).
You can, then, change the Docker file to match the environment for your deployment.

On the other hand, if you are distributing the program under different conditions as a binary,
it might be a good idea to have it compile as a static standalone binary.
To do that, all you have to run is the compile command for that.
You'll need to have Docker and just installed in your system.

just build-image
A docker image is created under the name of name:version.
If you didn't change the default values, it should be glyphos:1.0.0.
To run this image, you can execute.

docker run glyphos:1.0.0
If you have a Docker hub account, you can push your image to the docker hub cloud.
First, you should edit the dockerhub_org variable in the justfile and then execute the push command.

just push-image
# Github Actions

Several workflows are included.
Some workflows require configuration, while some others could work out of the box.

audit.yml: runs a security audit check and generate a report.
build.yml: build your project and upload the artifact as a release.
codecov.yml: run codecov and upload coverage report.
lint.yml: run clippy and generate a lint report.
tests.yml: run tests on github actions.
toc.yml: This generate a Table of Content for the Readme page, and could be removed.
