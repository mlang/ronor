#[macro_use]
extern crate clap;

use clap::{Arg, App, SubCommand};
use oauth2::AuthorizationCode;
use ronor::{Sonos, Player, Playlist};
use rustyline::Editor;
use std::process::{Command, Stdio};
use std::convert::TryFrom;
use url::Url;
use xdg::BaseDirectories;

#[macro_use]
extern crate error_chain;

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

fn main() -> Result<()> {
  let mut sonos = Sonos::try_from(BaseDirectories::with_prefix("ronor")?)?;
  let players = player_names(&mut sonos)?;
  let players: Vec<&str> = players.iter().map(|x| x.as_str()).collect();
  let matches = App::new(crate_name!())
    .author(crate_authors!())
    .version(crate_version!())
    .about("Sonos smart speaker controller")
    .subcommand(SubCommand::with_name("login")
      .about("Login with your sonos user account")
    ).subcommand(SubCommand::with_name("get-playlists")
      .about("Get list of playlists")
    ).subcommand(SubCommand::with_name("get-playlist")
      .about("Get list of tracks contained in a playlist")
      .arg(Arg::with_name("PLAYLIST").required(true))
    ).subcommand(SubCommand::with_name("get-groups")
      .about("Get list of groups")
    ).subcommand(SubCommand::with_name("load-audio-clip")
      .about("Schedule an audio clip for playback")
      .arg(Arg::with_name("NAME")
             .short("n").long("name").takes_value(true))
      .arg(Arg::with_name("APP_ID")
             .short("i").long("app-id").takes_value(true))
      .arg(Arg::with_name("PLAYER").required(true).help("Name of the player").possible_values(players.as_slice()))
      .arg(Arg::with_name("URL").required(true).help("Location of the audio clip"))
    ).subcommand(SubCommand::with_name("speak")
      .about("Send synthetic speech to a player")
      .arg(Arg::with_name("LANGUAGE")
             .short("l").long("language").takes_value(true).default_value("en"))
      .arg(Arg::with_name("WORDS_PER_MINUTE")
             .short("s").long("speed").takes_value(true).default_value("250"))
      .arg(Arg::with_name("VOLUME")
             .short("v").long("volume").takes_value(true).default_value("75"))
      .arg(Arg::with_name("PLAYER").required(true)
             .help("Name of the speaker")
	     .possible_values(players.as_slice()))
    ).subcommand(SubCommand::with_name("toggle-play-pause")
      .about("Toggle the playback state of the given group")
      .arg(Arg::with_name("GROUP"))
    ).subcommand(SubCommand::with_name("play")
      .about("Start playback for the given group")
      .arg(Arg::with_name("GROUP"))
    ).subcommand(SubCommand::with_name("pause")
      .about("Pause playback for the given group")
      .arg(Arg::with_name("GROUP"))
    ).subcommand(SubCommand::with_name("get-playback-status")
      .about("Get Playback Status (DEBUG)")
      .arg(Arg::with_name("GROUP"))
    ).get_matches();

  match matches.subcommand() {
    ("login", Some(_login_matches)) => {
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
    },
    ("load-audio-clip", Some(load_audio_clip_matches)) => {
      let player_name = load_audio_clip_matches.value_of("PLAYER").unwrap();
      if let Some(player) = find_player_by_name(&mut sonos, player_name)? {
        let url = value_t!(load_audio_clip_matches, "URL", Url).unwrap();
        if url.has_host() {
          let _clip = sonos.load_audio_clip(&player,
            load_audio_clip_matches.value_of("APP_ID").unwrap_or("guru.blind"),
	    load_audio_clip_matches.value_of("NAME").unwrap_or("clip"),
	    None,
	    None,
	    None,
            None,
	    Some(&url)
          )?;
	} else {
	  println!("The URL you provided does not look like Sonos will be able to reach it");
	  std::process::exit(1);
	}
      } else {
        println!("Player not found: {}", player_name);
        std::process::exit(1);
      }
      Ok(())
    },
    ("speak", Some(speak_matches)) => {
      let player_name = speak_matches.value_of("PLAYER").unwrap();
      if let Some(player) = find_player_by_name(&mut sonos, player_name)? {
        let mut args = vec![ String::from("-w)")
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
      } else {
        println!("Player not found: {}", player_name);
        std::process::exit(1);
      }
      Ok(())
    },
    ("get-playback-status", Some(get_playback_status_matches)) => {
      if !sonos.is_authorized() {
        println!("Not authroized, can not refresh");
      } else {
        let mut found = false;
        for household in sonos.get_households()?.iter() {
          for group in sonos.get_groups(&household)?.groups.iter() {
            if match get_playback_status_matches.value_of("GROUP") {
                 None => true,
                 Some(name) => name == group.name
               } {
              let playback_status = sonos.get_playback_status(&group)?;
              found = true;
              println!("'{}' = {:?}", group.name, playback_status);
            }
          }
        }
        if !found {
	  println!("The specified group was not found");
	  std::process::exit(1);
	}
      }
      Ok(())
    },
    ("get-groups", Some(_get_groups_matches)) => {
      if !sonos.is_authorized() {
        println!("Not authroized, can not refresh");
      } else {
        for household in sonos.get_households()?.iter() {
          for group in sonos.get_groups(&household)?.groups.iter() {
            println!("{}", group.name);
          }
        }
      }
      Ok(())
    },
    ("get-playlists", Some(_get_playlists_matches)) => {
      if !sonos.is_authorized() {
        println!("Not authroized, can not refresh");
      } else {
        for household in sonos.get_households()?.iter() {
          for playlist in sonos.get_playlists(&household)?.playlists.iter() {
            println!("{}", playlist.name);
          }
        }
      }
      Ok(())
    },
    ("get-playlist", Some(get_playlist_matches)) => {
      let playlist_name = get_playlist_matches.value_of("PLAYLIST").unwrap();
      for household in sonos.get_households()?.iter() {
        if let Some(playlist) = find_playlist_by_name(&mut sonos, playlist_name)? {
          for track in sonos.get_playlist(&household, &playlist)?.tracks.iter() {
            match &track.album {
	      Some(album) => println!("{} - {} - {}", &track.name, &track.artist, album),
  	      None => println!("{} - {}", &track.name, &track.artist)
	    }
  	  }
        }
      }
      Ok(())
    },
    ("toggle-play-pause", Some(toggle_play_pause_matches)) => {
      if !sonos.is_authorized() {
        println!("Not authroized, can not refresh");
      } else {
        for household in sonos.get_households()?.iter() {
          for group in sonos.get_groups(&household)?.groups.iter() {
            if match toggle_play_pause_matches.value_of("GROUP") {
                 None => true,
                 Some(name) => name == group.name
               } {
              sonos.toggle_play_pause(&group)?;
            }
          }
        }
      }
      Ok(())
    },
    ("play", Some(play_matches)) => {
      if !sonos.is_authorized() {
        println!("Not authroized, can not refresh");
      } else {
        for household in sonos.get_households()?.iter() {
          for group in sonos.get_groups(&household)?.groups.iter() {
            if match play_matches.value_of("GROUP") {
                 None => true,
                 Some(name) => name == group.name
               } {
              sonos.play(&group)?;
            }
          }
        }
      }
      Ok(())
    },
    ("pause", Some(pause_matches)) => {
      if !sonos.is_authorized() {
        println!("Not authroized, can not refresh");
      } else {
        for household in sonos.get_households()?.iter() {
          for group in sonos.get_groups(&household)?.groups.iter() {
            if match pause_matches.value_of("GROUP") {
                 None => true,
                 Some(name) => name == group.name
               } {
              sonos.pause(&group)?;
            }
          }
        }
      }
      Ok(())
    },
    _ => unreachable!()
  }
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
    players.extend(sonos.get_groups(&household)?.players.into_iter().map(|p| p.name.clone()));
  }
  Ok(players)
}
