# Sonos smart speaker controller API and CLI

Linux: [![Build Status](https://travis-ci.org/mlang/ronor.svg?branch=master)](https://travis-ci.org/mlang/ronor)

This project implements (most of) the [Sonos control API] in a rust crate. It also provides a simple command-line tool which can be used in scripts.

## Building

You likely need a recent rust compiler.  If you don't have `rustup` installed yet, I recommend you do so:

```console
$ curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

Now you are ready to build/install `ronor`:

```console
$ cargo install --git https://github.com/mlang/ronor
```

This will copy the binary to `~/.cargo/bin/ronor` which should be in your `PATH` if you are using `rustup`.

## Configuration

You have to register a developer account on integration.sonos.com and create your own integration point. You also need to create your own redirection endpoint on the web. A minimalistic example script is provided in [`static/sonos.php`].  Copy that file to a web space you control, and use it as the redirection URL required when you create the integration.

With your integration information ready, run `ronor init` and your client id, secret, and redirection url will be saved to `~/.config/ronor/`.

Now you can authorize ronor to access households belonging to your Sonos user account by running `ronor login`.

## How to use

See `ronor help` for a list of available commands.

### Favorites and Playlists

Sonos has two mechanisms for managing content you often play.  Favorites can be thought of as pointers to specific streaming service content.  For instance, a radio station, podcast, or a specific artist or album on a registered streaming service.  A playlist is a list of several tracks, possibly on different streaming services.  There is currently no API to create these, you have to use a Sonos controller like the iOS App to create favorites and playlists.

However, you can query and play favorites and playlists:

```console
$ ronor get-favorites
Das Soundportal Radio
Freies Radio Salzkammergut
Österreich 1
Radio FM4
Radio Helsinki
Radio Swiss Classic
radiOzora Chill channel
SRF 2 Kultur
$ ronor load-favorite --play 'Österreich 1' Schlafzimmer
$ ronor get-playlists
Acid
Psybient
PsyDub
$ ronor load-playlist --shuffle --crossfade --play PsyDub Wohnzimmer
```

### Managing groups

Use the [`modify-group`] subcommand to manage grouping of logical players.

For example, imagine the following household of three players and no grouping.

```console
$ ronor inventory
Bad = Bad
Wohnzimmer = Wohnzimmer
Schlafzimmer = Schlafzimmer
```

That means, each player is the sole member of a group with the same name.

Now lets make a group of `Schlafzimmer` (bedroom) and `Bad` (Bathroom).

```console
$ ronor modify-group Schlafzimmer --add Bad
Schlafzimmer -> Schlafzimmer + 1
$ ronor inventory
Schlafzimmer + 1 = Schlafzimmer + Bad
Wohnzimmer = Wohnzimmer
```

To undo this group, we simply remove `Bad` from `Schlafzimmer + 1` again:

```console
$ ronor modify-group 'Schlafzimmer + 1' --remove Bad
Schlafzimmer + 1 -> Schlafzimmer
$ ronor inventory
Bad = Bad
Wohnzimmer = Wohnzimmer
Schlafzimmer = Schlafzimmer
```

Notice that you never have to name groups.  Sonos will automatically choose a name for a newly created group based on the coordinating player and the number of other members.

### Text to speech

For the text-to-speech functionality (`ronor speak`) you need `espeak` and `ffmpeg` installed. Simply pipe text to `STDIN` and it should be spoken by the desired player.

```console
$ echo "Hallo Wohnzimmer"|ronor speak --language de Wohnzimmer
```

Alternatively, `ronor speak` can scrape predefined web resources and speak the extracted text.  The following command will speak the current weather forecast for Styria in Austria:

```console
$ ronor speak --scrape wetter.orf.at/steiermark Wohnzimmer
```

The following scraping sources are predefined:

* Weather in Austria
 * ORF
  * [wetter.orf.at/burgenland]
  * [wetter.orf.at/kaernten]
  * [wetter.orf.at/niederoesterreich]
  * [wetter.orf.at/oberoesterreich]
  * [wetter.orf.at/salzburg]
  * [wetter.orf.at/steiermark]
  * [wetter.orf.at/tirol]
  * [wetter.orf.at/vorarlberg]
  * [wetter.orf.at/wien]
 * ZAMG
  * [zamg.ac.at/cms/de/wetter/wetter-oesterreich/burgenland]
  * [zamg.ac.at/cms/de/wetter/wetter-oesterreich/kaernten]
  * [zamg.ac.at/cms/de/wetter/wetter-oesterreich/niederoesterreich]
  * [zamg.ac.at/cms/de/wetter/wetter-oesterreich/oberoesterreich]
  * [zamg.ac.at/cms/de/wetter/wetter-oesterreich/salzburg]
  * [zamg.ac.at/cms/de/wetter/wetter-oesterreich/steiermark]
  * [zamg.ac.at/cms/de/wetter/wetter-oesterreich/tirol]
  * [zamg.ac.at/cms/de/wetter/wetter-oesterreich/vorarlberg]
  * [zamg.ac.at/cms/de/wetter/wetter-oesterreich/wien]

`ronor speak` makes use of [transfer.sh] for temporary storage and the `loadAudioClip` API.  If you'd like to play already prepared audio clips, use `ronor load-audio-clip`.

[Sonos control API]: https://developer.sonos.com/reference/control-api/
[transfer.sh]: https://transfer.sh/
[`static/sonos.php`]: https://github.com/mlang/ronor/blob/master/static/sonos.php
[`modify-group`]: https://github.com/mlang/ronor/blob/master/src/subcmds/modify_group.rs
[wetter.orf.at/burgenland]: https://wetter.orf.at/burgenland/prognose
[wetter.orf.at/kaernten]: https://wetter.orf.at/kaernten/prognose
[wetter.orf.at/niederoesterreich]: https://wetter.orf.at/niederoesterreich/prognose
[wetter.orf.at/oberoesterreich]: https://wetter.orf.at/oberoesterreich/prognose
[wetter.orf.at/salzburg]: https://wetter.orf.at/salzburg/prognose
[wetter.orf.at/steiermark]: https://wetter.orf.at/steiermark/prognose
[wetter.orf.at/tirol]: https://wetter.orf.at/tirol/prognose
[wetter.orf.at/vorarlberg]: https://wetter.orf.at/vorarlberg/prognose
[wetter.orf.at/wien]: https://wetter.orf.at/wien/prognose
[zamg.ac.at/cms/de/wetter/wetter-oesterreich/burgenland]: https://www.zamg.ac.at/cms/de/wetter/wetter-oesterreich/burgenland/
[zamg.ac.at/cms/de/wetter/wetter-oesterreich/kaernten]: https://www.zamg.ac.at/cms/de/wetter/wetter-oesterreich/kaernten/
[zamg.ac.at/cms/de/wetter/wetter-oesterreich/niederoesterreich]: https://www.zamg.ac.at/cms/de/wetter/wetter-oesterreich/niederoesterreich/
[zamg.ac.at/cms/de/wetter/wetter-oesterreich/oberoesterreich]: https://www.zamg.ac.at/cms/de/wetter/wetter-oesterreich/oberoesterreich/
[zamg.ac.at/cms/de/wetter/wetter-oesterreich/salzburg]: https://www.zamg.ac.at/cms/de/wetter/wetter-oesterreich/salzburg/
[zamg.ac.at/cms/de/wetter/wetter-oesterreich/steiermark]: https://www.zamg.ac.at/cms/de/wetter/wetter-oesterreich/steiermark/
[zamg.ac.at/cms/de/wetter/wetter-oesterreich/tirol]: https://www.zamg.ac.at/cms/de/wetter/wetter-oesterreich/tirol/
[zamg.ac.at/cms/de/wetter/wetter-oesterreich/vorarlberg]: https://www.zamg.ac.at/cms/de/wetter/wetter-oesterreich/vorarlberg/
[zamg.ac.at/cms/de/wetter/wetter-oesterreich/wien]: https://www.zamg.ac.at/cms/de/wetter/wetter-oesterreich/wien/
