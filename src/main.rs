#[macro_use]
extern crate clap;

#[macro_use]
extern crate error_chain;

use clap::{Arg, ArgMatches, App, SubCommand};
use oauth2::{AuthorizationCode, ClientId, ClientSecret, RedirectUrl};
use ronor::{Sonos, Favorite, Group, PlaybackState, Player, Playlist};
use rustyline::Editor;
use std::process::{Command, Stdio, exit};
use std::convert::TryFrom;
use url::Url;
use xdg::BaseDirectories;

error_chain! {
  links {
    API(ronor::Error, ronor::ErrorKind);
  }
  foreign_links {
    IO(std::io::Error);
    XDG(xdg::BaseDirectoriesError);
    ReadLine(rustyline::error::ReadlineError);
    ParseInt(std::num::ParseIntError);
    UrlParse(url::ParseError);
  }
}

fn build_cli() -> App<'static, 'static> {
  App::new(crate_name!())
    .author(crate_authors!())
    .version(crate_version!())
    .about("Sonos smart speaker controller")
    .subcommand(SubCommand::with_name("init")
      .about("Initialise sonos integration configuration")
    ).subcommand(SubCommand::with_name("login")
      .about("Login with your sonos user account and authorize ronor")
 // ).subcommand(SubCommand::with_name("completions")
 //   .about("Generates completion scripts for your shell")
 //   .arg(Arg::with_name("SHELL")
 //          .required(true)
 // 	     .possible_values(&["bash", "fish", "zsh"])
 //          .help("The shell to generate the script for"))
    ).subcommand(SubCommand::with_name("get-playlists")
      .about("Get list of playlists")
    ).subcommand(SubCommand::with_name("get-playlist")
      .about("Get list of tracks contained in a playlist")
      .arg(Arg::with_name("PLAYLIST").required(true))
    ).subcommand(SubCommand::with_name("get-favorites")
      .about("Get list of favorites")
    ).subcommand(SubCommand::with_name("get-groups")
      .about("Get list of groups")
    ).subcommand(SubCommand::with_name("get-players")
      .about("Get list of players")
    ).subcommand(SubCommand::with_name("get-group-volume")
      .about("Get group volume")
      .arg(Arg::with_name("GROUP"))
    ).subcommand(SubCommand::with_name("set-group-volume")
      .about("Set group volume")
      .arg(Arg::with_name("GROUP").required(true))
      .arg(Arg::with_name("VOLUME").required(true).help("Volume (0-100)"))
    ).subcommand(SubCommand::with_name("get-player-volume")
      .about("Get player volume")
      .arg(Arg::with_name("PLAYER"))
    ).subcommand(SubCommand::with_name("set-player-volume")
      .about("Set player volume")
      .arg(Arg::with_name("PLAYER").required(true))
      .arg(Arg::with_name("VOLUME").required(true).help("Volume (0-100)"))
    ).subcommand(SubCommand::with_name("load-audio-clip")
      .about("Schedule an audio clip for playback")
      .arg(Arg::with_name("NAME")
             .short("n").long("name").takes_value(true))
      .arg(Arg::with_name("APP_ID")
             .short("i").long("app-id").takes_value(true))
      .arg(Arg::with_name("PLAYER")
             .required(true)
             .help("Name of the player"))
      .arg(Arg::with_name("URL")
             .required(true)
             .help("Location of the audio clip"))
    ).subcommand(SubCommand::with_name("load-home-theater-playback")
      .about("Signal a player to switch to its TV input (optical or HDMI)")
      .arg(Arg::with_name("PLAYER")
             .required(true)
             .help("Name of the player"))
    ).subcommand(SubCommand::with_name("load-line-in")
      .about("Change the given group to the line-in source of a specified player")
      .arg(Arg::with_name("GROUP")
             .required(true)
             .help("Name of the group"))
      .arg(Arg::with_name("PLAYER")
             .help("Name of the player"))
    ).subcommand(SubCommand::with_name("speak")
      .about("Send synthetic speech to a player")
      .arg(Arg::with_name("LANGUAGE")
             .short("l").long("language").takes_value(true)
             .default_value("en"))
      .arg(Arg::with_name("WORDS_PER_MINUTE")
             .short("s").long("speed").takes_value(true).default_value("250"))
      .arg(Arg::with_name("VOLUME")
             .short("v").long("volume").takes_value(true).default_value("75"))
      .arg(Arg::with_name("PLAYER").required(true)
             .help("Name of the speaker"))
    ).subcommand(SubCommand::with_name("load-favorite")
      .about("Play the specified favorite in the given group")
      .arg(Arg::with_name("FAVORITE").required(true))
      .arg(Arg::with_name("GROUP").required(true))
    ).subcommand(SubCommand::with_name("load-playlist")
      .about("Play the specified playlist in the given group")
      .arg(Arg::with_name("PLAYLIST").required(true))
      .arg(Arg::with_name("GROUP").required(true))
    ).subcommand(SubCommand::with_name("toggle-play-pause")
      .about("Toggle the playback state of the given group")
      .arg(Arg::with_name("GROUP"))
    ).subcommand(SubCommand::with_name("play")
      .about("Start playback for the given group")
      .arg(Arg::with_name("GROUP"))
    ).subcommand(SubCommand::with_name("pause")
      .about("Pause playback for the given group")
      .arg(Arg::with_name("GROUP"))
    ).subcommand(SubCommand::with_name("skip-to-previous-track")
      .about("Go to previous track in the given group")
      .arg(Arg::with_name("GROUP").required(true))
    ).subcommand(SubCommand::with_name("skip-to-next-track")
      .about("Go to next track in the given group")
      .arg(Arg::with_name("GROUP").required(true))
    ).subcommand(SubCommand::with_name("get-playback-status")
      .about("Get playback status (DEBUG)")
      .arg(Arg::with_name("GROUP"))
    ).subcommand(SubCommand::with_name("get-metadata-status")
      .about("Get playback status (DEBUG)")
      .arg(Arg::with_name("GROUP"))
    ).subcommand(SubCommand::with_name("now-playing")
      .about("Display information about currently playing tracks")
      .arg(Arg::with_name("GROUP"))
    )
}

fn main() -> Result<()> {
  let mut sonos = Sonos::try_from(BaseDirectories::with_prefix("ronor")?)?;
  //let players = player_names(&mut sonos)?;
  //let players: Vec<&str> = players.iter().map(|x| x.as_str()).collect();
  match build_cli().get_matches().subcommand() {
    ("init", Some(matches)) =>
      init(&mut sonos, matches),
    ("login", Some(matches)) =>
      login(&mut sonos, matches),
    ("completions", Some(matches)) => {
      let shell = matches.value_of("SHELL").unwrap();
      build_cli().gen_completions_to(
        "ronor",
	shell.parse::<>().unwrap(),
	&mut std::io::stdout()
      );
      Ok(())
    },
    ("load-audio-clip", Some(matches)) =>
      load_audio_clip(&mut sonos, matches),
    ("load-home-theater-playback", Some(matches)) =>
      load_home_theater_playback(&mut sonos, matches),
    ("load-line-in", Some(matches)) =>
      load_line_in(&mut sonos, matches),
    ("speak", Some(matches)) =>
      speak(&mut sonos, matches),
    ("load-favorite", Some(matches)) =>
      load_favorite(&mut sonos, matches),
    ("load-playlist", Some(matches)) =>
      load_playlist(&mut sonos, matches),
    ("get-playback-status", Some(matches)) =>
      get_playback_status(&mut sonos, matches),
    ("get-metadata-status", Some(matches)) =>
      get_metadata_status(&mut sonos, matches),
    ("get-group-volume", Some(matches)) =>
      get_group_volume(&mut sonos, matches),
    ("set-group-volume", Some(matches)) =>
      set_group_volume(&mut sonos, matches),
    ("get-player-volume", Some(matches)) =>
      get_player_volume(&mut sonos, matches),
    ("set-player-volume", Some(matches)) =>
      set_player_volume(&mut sonos, matches),
    ("get-groups", Some(matches)) =>
      get_groups(&mut sonos, matches),
    ("get-players", Some(matches)) =>
      get_players(&mut sonos, matches),
    ("get-playlists", Some(matches)) =>
      get_playlists(&mut sonos, matches),
    ("get-playlist", Some(matches)) =>
      get_playlist(&mut sonos, matches),
    ("get-favorites", Some(matches)) =>
      get_favorites(&mut sonos, matches),
    ("toggle-play-pause", Some(matches)) =>
      toggle_play_pause(&mut sonos, matches),
    ("play", Some(matches)) =>
      play(&mut sonos, matches),
    ("pause", Some(matches)) =>
      pause(&mut sonos, matches),
    ("skip-to-previous-track", Some(matches)) =>
      skip_to_previous_track(&mut sonos, matches),
    ("skip-to-next-track", Some(matches)) =>
      skip_to_next_track(&mut sonos, matches),
    ("now-playing", Some(matches)) =>
     now_playing(&mut sonos, matches),
    _ => unreachable!()
  }
}

fn init(sonos: &mut Sonos, _matches: &ArgMatches) -> Result<()> {
  println!("Go to https://integration.sonos.com/ and create an account.");
  println!("");
  println!("Create a new control integration.");
  println!("");
  let mut console = Editor::<()>::new();
  let client_id = ClientId::new(console.readline("Client identifier: ")?);
  let client_secret = ClientSecret::new(console.readline("Client secret: ")?);
  let redirect_url = RedirectUrl::new(
    Url::parse(&console.readline("Redirection URL: ")?)?
  );
  sonos.set_integration_config(client_id, client_secret, redirect_url)?;
  println!("");
  println!("OK, we're ready to go.");
  println!("Now run 'ronor login' to authorize access to your Sonos user account.");
  Ok(())
}

fn login(sonos: &mut Sonos, _matches: &ArgMatches) -> Result<()> {
  let (auth_url, csrf_token) = sonos.authorization_url()?;
  let _lynx = Command::new("lynx")
    .arg("-nopause")
    .arg(auth_url.as_str())
    .status().expect("Failed to fire up browser.");
  println!("Token: {}", csrf_token.secret());
  let mut console = Editor::<()>::new();
  let code = console.readline("Code: ")?;
  sonos.authorize(AuthorizationCode::new(code.trim().to_string()))?;
  Ok(())
}

macro_rules! with_authorization {
  ($sonos:ident, $code:block) => {
    if !$sonos.is_authorized() {
      return Err("Not authorized".into());
    } else $code
  };
}

macro_rules! with_favorite {
  ($sonos:ident, $matches:ident, $favorite:ident, $code:block) => {
    if let Some($favorite) = find_favorite_by_name(
      $sonos, $matches.value_of("FAVORITE").unwrap()
    )? $code
    else {
      return Err("Favorite not found".into());
    }
  }
}

macro_rules! with_group {
  ($sonos:ident, $matches:ident, $group:ident, $code:block) => {
    if let Some($group) = find_group_by_name(
      $sonos, $matches.value_of("GROUP").unwrap()
    )? $code
    else {
      return Err("Group not found".into());
    }
  }
}

macro_rules! with_player {
  ($sonos:ident, $matches:ident, $player:ident, $code:block) => {
    if let Some($player) = find_player_by_name(
      $sonos, $matches.value_of("PLAYER").unwrap()
    )? $code
    else {
      return Err("Player not found".into());
    }
  }
}

macro_rules! with_playlist {
  ($sonos:ident, $matches:ident, $playlist:ident, $code:block) => {
    if let Some($playlist) = find_playlist_by_name(
      $sonos, $matches.value_of("PLAYLIST").unwrap()
    )? $code
    else {
      return Err("Playlist not found".into());
    }
  }
}

fn load_audio_clip(sonos: &mut Sonos, matches: &ArgMatches) -> Result<()> {
  with_authorization!(sonos, {
    with_player!(sonos, matches, player, {
      let url = value_t!(matches, "URL", Url).unwrap();
      if url.has_host() {
        sonos.load_audio_clip(&player,
          matches.value_of("APP_ID").unwrap_or("guru.blind"),
          matches.value_of("NAME").unwrap_or("clip"),
          None,
          None,
          None,
          None,
          Some(&url)
        )?;
      } else {
        println!("The URL you provided does not look like Sonos will be able to reach it");
        exit(1);
      }
      Ok(())
    })
  })
}

fn load_home_theater_playback(sonos: &mut Sonos, matches: &ArgMatches) -> Result<()> {
  with_authorization!(sonos, {
    with_player!(sonos, matches, player, {
      Ok(sonos.load_home_theater_playback(&player)?)
    })
  })
}

fn speak(sonos: &mut Sonos, matches: &ArgMatches) -> Result<()> {
  with_authorization!(sonos, {
    with_player!(sonos, matches, player, {
      let mut args = vec![ String::from("-w")
                         , String::from("/dev/stdout")
                         , String::from("--stdin")];
      if let Some(language) = matches.value_of("LANGUAGE") {
        args.extend(vec![String::from("-v"), language.to_string()]);
      }
      if let Some(wpm) = matches.value_of("WORDS_PER_MINUTE") {
        args.extend(vec![String::from("-s"), wpm.to_string()]);
      }
      if let Some(volume) = matches.value_of("VOLUME") {
        let volume = volume.parse::<u8>()? * 2;
        args.extend(vec![String::from("-a"), volume.to_string()]);
      }
      let espeak = Command::new("espeak")
        .args(args)
        .stdout(Stdio::piped()).spawn()?;
      if let Some(stdout) = espeak.stdout {
        let ffmpeg = Command::new("ffmpeg")
          .args(&["-i", "-", "-v", "fatal", "-b:a", "96k", "-f", "mp3", "-"])
          .stdin(stdout).stdout(Stdio::piped()).spawn()?;
        if let Some(stdout) = ffmpeg.stdout {
          let curl = Command::new("curl")
            .args(&["--upload-file", "-", "https://transfer.sh/espeak.mp3"])
            .stdin(stdout).output()?;
          if curl.status.success() {
            let url = Url::parse(&String::from_utf8_lossy(&curl.stdout))?;
            let _clip = sonos.load_audio_clip(&player,
              "guru.blind",
              "ping",
              None,
              None,
              None,
              None,
              Some(&url)
            )?;
          }
        }
      }
      Ok(())
    })
  })
}

fn get_group_volume(sonos: &mut Sonos, matches: &ArgMatches) -> Result<()> {
  with_authorization!(sonos, {
    let mut found = false;
    for household in sonos.get_households()?.iter() {
      for group in sonos.get_groups(&household)?.groups.iter() {
        if matches.value_of("GROUP").map_or(true, |name| name == group.name) {
          found = true;
          let group_volume = sonos.get_group_volume(&group)?;
          println!("'{}' = {:?}", group.name, group_volume);
        }
      }
    }
    if !found {
      println!("The specified group was not found");
      exit(1);
    }
    Ok(())
  })
}

fn set_group_volume(sonos: &mut Sonos, matches: &ArgMatches) -> Result<()> {
  with_authorization!(sonos, {
    with_group!(sonos, matches, group, {
      let volume = matches.value_of("VOLUME").unwrap();
      Ok(sonos.set_group_volume(&group, volume.parse::<>()?)?)
    })
  })
}

fn get_player_volume(sonos: &mut Sonos, matches: &ArgMatches) -> Result<()> {
  with_authorization!(sonos, {
    let mut found = false;
    for household in sonos.get_households()?.iter() {
      for player in sonos.get_groups(&household)?.players.iter().filter(|player|
        matches.value_of("PLAYER").map_or(true, |name| name == player.name)
      ) {
        found = true;
        let player_volume = sonos.get_player_volume(&player)?;
        println!("'{}' = {:?}", player.name, player_volume);
      }
    }
    if !found {
      println!("The specified player was not found");
      exit(1);
    }
    Ok(())
  })
}

fn set_player_volume(sonos: &mut Sonos, matches: &ArgMatches) -> Result<()> {
  with_authorization!(sonos, {
    with_player!(sonos, matches, player, {
      let volume = matches.value_of("VOLUME").unwrap();
      Ok(sonos.set_player_volume(&player, volume.parse::<>()?)?)
    })
  })
}

fn get_playback_status(sonos: &mut Sonos, matches: &ArgMatches) -> Result<()> {
  with_authorization!(sonos, {
    let mut found = false;
    for household in sonos.get_households()?.iter() {
      for group in sonos.get_groups(&household)?.groups.iter() {
        if matches.value_of("GROUP").map_or(true, |name| name == group.name) {
          found = true;
          let playback_status = sonos.get_playback_status(&group)?;
          println!("'{}' = {:?}", group.name, playback_status);
        }
      }
    }
    if !found {
      println!("The specified group was not found");
      exit(1);
    }
    Ok(())
  })
}

fn get_metadata_status(sonos: &mut Sonos, matches: &ArgMatches) -> Result<()> {
  with_authorization!(sonos, {
    let mut found = false;
    for household in sonos.get_households()?.iter() {
      for group in sonos.get_groups(&household)?.groups.iter() {
        if matches.value_of("GROUP").map_or(true, |name| name == group.name) {
          found = true;
          let metadata_status = sonos.get_metadata_status(&group)?;
          println!("'{}' = {:?}", group.name, metadata_status);
        }
      }
    }
    if !found {
      println!("The specified group was not found");
      exit(1);
    }
    Ok(())
  })
}

fn get_groups(sonos: &mut Sonos, _matches: &ArgMatches) -> Result<()> {
  with_authorization!(sonos, {
    for household in sonos.get_households()?.iter() {
      for group in sonos.get_groups(&household)?.groups.iter() {
        println!("{}", group.name);
      }
    }
    Ok(())
  })
}

fn get_players(sonos: &mut Sonos, _matches: &ArgMatches) -> Result<()> {
  with_authorization!(sonos, {
    for household in sonos.get_households()?.iter() {
      for player in sonos.get_groups(&household)?.players.iter() {
        println!("{}", player.name);
      }
    }
    Ok(())
  })
}

fn get_playlists(sonos: &mut Sonos, _matches: &ArgMatches) -> Result<()> {
  with_authorization!(sonos, {
    for household in sonos.get_households()?.iter() {
      for playlist in sonos.get_playlists(&household)?.playlists.iter() {
        println!("{}", playlist.name);
      }
    }
    Ok(())
  })
}

fn get_playlist(sonos: &mut Sonos, matches: &ArgMatches) -> Result<()> {
  with_authorization!(sonos, {
    with_playlist!(sonos, matches, playlist, {
      for household in sonos.get_households()?.iter() {
        for track in sonos.get_playlist(&household, &playlist)?.tracks.iter() {
          match &track.album {
            Some(album) => println!("{} - {} - {}",
                                    &track.name, &track.artist, album),
            None => println!("{} - {}",
                             &track.name, &track.artist)
          }
        }
      }
      Ok(())
    })
  })
}

fn load_favorite(sonos: &mut Sonos, matches: &ArgMatches) -> Result<()> {
  with_authorization!(sonos, {
    with_favorite!(sonos, matches, favorite, {
      with_group!(sonos, matches, group, {
        Ok(sonos.load_favorite(&group, &favorite, true, None)?)
      })
    })
  })
}

fn load_line_in(sonos: &mut Sonos, matches: &ArgMatches) -> Result<()> {
  with_authorization!(sonos, {
    with_group!(sonos, matches, group, {
      Ok(match matches.value_of("PLAYER") {
        None => sonos.load_line_in(&group, None, true)?,
	Some(player_name) => match find_player_by_name(sonos, player_name) {
          Ok(player) => match player {
	    Some(player) => sonos.load_line_in(&group, Some(&player), true)?,
	    None => sonos.load_line_in(&group, None, true)?
	  }
	  Err(_) => sonos.load_line_in(&group, None, true)?
        }
      })
    })
  })
}

fn load_playlist(sonos: &mut Sonos, matches: &ArgMatches) -> Result<()> {
  with_authorization!(sonos, {
    with_playlist!(sonos, matches, playlist, {
      with_group!(sonos, matches, group, {
        Ok(sonos.load_playlist(&group, &playlist, true, None)?)
      })
    })
  })
}

fn get_favorites(sonos: &mut Sonos, _matches: &ArgMatches) -> Result<()> {
  with_authorization!(sonos, {
    for household in sonos.get_households()?.iter() {
      for favorite in sonos.get_favorites(&household)?.items.iter() {
        println!("{}", favorite.name);
      }
    }
    Ok(())
  })
}

fn toggle_play_pause(sonos: &mut Sonos, matches: &ArgMatches) -> Result<()> {
  with_authorization!(sonos, {
    let mut found = false;
    for household in sonos.get_households()?.iter() {
      for group in sonos.get_groups(&household)?.groups.iter() {
        if matches.value_of("GROUP").map_or(true, |name| name == group.name) {
          found = true;
          sonos.toggle_play_pause(&group)?;
        }
      }
    }
    if !found {
      println!("Group not found");
      exit(1);
    }
    Ok(())
  })
}

fn play(sonos: &mut Sonos, matches: &ArgMatches) -> Result<()> {
  with_authorization!(sonos, {
    let mut found = false;
    for household in sonos.get_households()?.iter() {
      for group in sonos.get_groups(&household)?.groups.iter() {
        if matches.value_of("GROUP").map_or(true, |name| name == group.name) {
          found = true;
          sonos.play(&group)?;
        }
      }
    }
    if !found {
      println!("Group not found");
      exit(1);
    }
    Ok(())
  })
}

fn pause(sonos: &mut Sonos, matches: &ArgMatches) -> Result<()> {
  with_authorization!(sonos, {
    let mut found = false;
    for household in sonos.get_households()?.iter() {
      for group in sonos.get_groups(&household)?.groups.iter() {
        if matches.value_of("GROUP").map_or(true, |name| name == group.name) {
          found = true;
          sonos.pause(&group)?;
        }
      }
    }
    if !found {
      println!("Group not found");
      exit(1);
    }
    Ok(())
  })
}

fn skip_to_previous_track(sonos: &mut Sonos, matches: &ArgMatches) -> Result<()> {
  with_authorization!(sonos, {
    with_group!(sonos, matches, group, {
      Ok(sonos.skip_to_previous_track(&group)?)
    })
  })
}

fn skip_to_next_track(sonos: &mut Sonos, matches: &ArgMatches) -> Result<()> {
  with_authorization!(sonos, {
    with_group!(sonos, matches, group, {
      Ok(sonos.skip_to_next_track(&group)?)
    })
  })
}

fn now_playing(sonos: &mut Sonos, matches: &ArgMatches) -> Result<()> {
  with_authorization!(sonos, {
    let mut found = false;
    for household in sonos.get_households()?.iter() {
      for group in sonos.get_groups(&household)?.groups.iter().filter(|group|
        matches.value_of("GROUP").map_or(true, |name| name == group.name)
      ) {
        found = true;
        if group.playback_state == PlaybackState::Playing {
          let metadata_status = sonos.get_metadata_status(&group)?;
          let mut parts = Vec::new();
          if let Some(container) = &metadata_status.container {
            if let Some(name) = &container.name {
              parts.push(name.as_str());
            }
            if let Some(service) = &container.service {
              parts.push(service.name.as_str());
            }
          }
          if let Some(current_item) = &metadata_status.current_item {
            if let Some(name) = &current_item.track.name {
              parts.push(name.as_str());
            }
          }
          if let Some(stream_info) = &metadata_status.stream_info {
            parts.push(stream_info.trim());
          }
          let mut parts = parts.iter();
          if let Some(part) = parts.next() {
            print!("{:?} => {}", group.name, part);
            for part in parts {
              print!(" - {}", part);
            }
            println!();
          }
        }
      }
    }
    if matches.value_of("GROUP").is_some() && !found {
      println!("The specified group was not found");
      exit(1);
    }
    Ok(())
  })
}

fn find_group_by_name(
  sonos: &mut Sonos, name: &str
) -> Result<Option<Group>> {
  for household in sonos.get_households()?.into_iter() {
    for group in sonos.get_groups(&household)?.groups.into_iter() {
      if group.name == name {
        return Ok(Some(group))
      }
    }
  }
  Ok(None)
}

fn find_player_by_name(
  sonos: &mut Sonos, name: &str
) -> Result<Option<Player>> {
  for household in sonos.get_households()?.into_iter() {
    for player in sonos.get_groups(&household)?.players.into_iter() {
      if player.name == name {
        return Ok(Some(player))
      }
    }
  }
  Ok(None)
}

fn find_favorite_by_name(
  sonos: &mut Sonos, name: &str
) -> Result<Option<Favorite>> {
  for household in sonos.get_households()?.into_iter() {
    for favorite in sonos.get_favorites(&household)?.items.into_iter() {
      if favorite.name == name {
        return Ok(Some(favorite))
      }
    }
  }
  Ok(None)
}

fn find_playlist_by_name(
  sonos: &mut Sonos, name: &str
) -> Result<Option<Playlist>> {
  for household in sonos.get_households()?.into_iter() {
    for playlist in sonos.get_playlists(&household)?.playlists.into_iter() {
      if playlist.name == name {
        return Ok(Some(playlist))
      }
    }
  }
  Ok(None)
}

fn player_names(sonos: &mut Sonos) -> Result<Vec<String>> {
  let mut players = Vec::new();
  for household in sonos.get_households()?.into_iter() {
    players.extend(
      sonos.get_groups(&household)?.players.into_iter().map(|p| p.name)
    );
  }
  Ok(players)
}


