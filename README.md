# Sonos smart speaker controller API and CLI

This project implements (most of) the Sonos control API
in a rust crate.  It also provides a simple command-line tool which
can be used in scripts.

You likely need a recent rust compiler.

Build with "cargo build".

See src/main.rs for an example on how to use the API.

For the text-to-speech functionality (ronor speak) you need espeak and ffmpeg
installed.  Simply pipe text to STDIN and it should arrive at the desired player.

Documentation is very scarce right now.  Please refer to
developer.sonos.com for documentation about the underlying API calls.
I am not a native english speaker, and just copying text from
developer.sonos.com seems sort of wrong.  PRs very welcome.

