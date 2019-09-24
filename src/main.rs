#[macro_use]
extern crate clap;

use clap::{Arg, App, SubCommand};
use oauth2::AuthorizationCode;
use ronor::{IntegrationConfig, Sonos, Player};
use rustyline::Editor;
use std::process::{Command, Stdio};
use std::convert::TryFrom;
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
  }
}

fn main() -> Result<()> {
  let matches = App::new("ronor")
    .version(crate_version!())
    .about("Sonos smart speaker controller")
    .subcommand(SubCommand::with_name("login")
      .about("Login with your sonos user account")
    ).subcommand(SubCommand::with_name("refresh")
      .about("Refresh the access token")
    ).subcommand(SubCommand::with_name("speak")
      .about("Speak standard input")
      .arg(Arg::with_name("LANGUAGE")
             .short("l").long("language").takes_value(true).default_value("en"))
      .arg(Arg::with_name("WORDS_PER_MINUTE")
             .short("s").long("speed").takes_value(true).default_value("250"))
      .arg(Arg::with_name("VOLUME")
             .short("v").long("volume").takes_value(true).default_value("75"))
      .arg(Arg::with_name("PLAYER").required(true).help("Name of the speaker"))
    ).subcommand(SubCommand::with_name("togglePlayPause")
      .about("Toggle the playback state of the given group")
      .arg(Arg::with_name("GROUP"))
    ).subcommand(SubCommand::with_name("getPlaybackStatus")
      .about("Get Playback Status (DEBUG)")
      .arg(Arg::with_name("GROUP"))
    )
    .get_matches();
  let mut sonos = Sonos::try_from(BaseDirectories::with_prefix("ronor")?)?;
  if let Some(matches) = matches.subcommand_matches("login") {
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
  } else if let Some(matches) = matches.subcommand_matches("refresh") {
    if !sonos.is_registered() {
      println!("ronor is not registered, run authroize");
    } else if !sonos.is_authorized() {
      println!("Not authroized, can not refresh");
    } else {
      sonos.refresh_token()?;
    }
    Ok(())
  } else if let Some(matches) = matches.subcommand_matches("speak") {
    let player_name = matches.value_of("PLAYER").unwrap();
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
            let url = String::from_utf8_lossy(&curl.stdout);
	    let _clip = sonos.load_audio_clip(&player,
              String::from("guru.blind"), String::from("ping"), None, None, None,
              None, Some(url.to_string())
            )?;
	  }
        }
      }
    } else {
      println!("Player not found: {}", player_name);
      std::process::exit(1);
    }
    Ok(())
  } else if let Some(matches) = matches.subcommand_matches("getPlaybackStatus") {
    if !sonos.is_authorized() {
      println!("Not authroized, can not refresh");
    } else {
      for household in sonos.get_households()?.iter() {
        for group in sonos.get_groups(&household)?.groups.iter() {
          if match matches.value_of("GROUP") {
	       None => true,
	       Some(name) => name == group.name
	     } {
            let playback_status = sonos.get_playback_status(&group)?;
            println!("'{}' = {:?}", group.name, playback_status);
	  }
        }
      }
    }
    Ok(())
  } else if let Some(matches) = matches.subcommand_matches("togglePlayPause") {
    if !sonos.is_authorized() {
      println!("Not authroized, can not refresh");
    } else {
      for household in sonos.get_households()?.iter() {
        for group in sonos.get_groups(&household)?.groups.iter() {
          if match matches.value_of("GROUP") {
	       None => true,
	       Some(name) => name == group.name
	     } {
            sonos.toggle_play_pause(&group)?;
	  }
        }
      }
    }
    Ok(())
  } else {
    for household in sonos.get_households()?.iter() {
      for playlist in sonos.get_playlists(&household)?.playlists.iter() {
        println!("{:?}", sonos.get_playlist(&household, &playlist)?);
      }
    }
    Ok(())
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
