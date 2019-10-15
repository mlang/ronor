use clap::{Arg, ArgMatches, App};
use ronor::{Sonos, PlaybackState};
use super::{Result, ErrorKind};

pub const NAME: &str = "now-playing";

pub fn build() -> App<'static, 'static> {
  App::new(NAME).visible_alias("np")
    .about("Describes what is currently playing")
    .arg(Arg::with_name("GROUP"))
}

pub fn run(sonos: &mut Sonos, matches: &ArgMatches) -> Result<()> {
  let group_name = matches.value_of("GROUP");
  let mut found = false;
  for household in sonos.get_households()?.iter() {
    for group in sonos.get_groups(&household)?.groups.iter().filter(|group|
      group_name.map_or(true, |name| name == group.name)
    ) {
      found = true;
      if group.playback_state == PlaybackState::Playing {
        let metadata_status = sonos.get_metadata_status(&group)?;
        let mut parts = Vec::new();
        if let Some(container) = &metadata_status.container {
          if container.type_.is_some() && container.type_.as_ref().unwrap() == "linein.homeTheater" {
            parts.push("Home theater");
          } else {
            if let Some(name) = &container.name {
              parts.push(name.as_str());
            }
            if let Some(service) = &container.service {
              parts.push(service.name.as_str());
            }
          }
        }
        if let Some(current_item) = &metadata_status.current_item {
          if let Some(name) = &current_item.track.name {
            parts.push(name.as_str());
            if let Some(album) = &current_item.track.album {
              parts.push(album.name.as_str());
            }
            if let Some(artist) = &current_item.track.artist {
              parts.push(artist.name.as_str());
            }
            parts.push(current_item.track.service.name.as_str());
          }
        }
        if let Some(stream_info) = &metadata_status.stream_info {
          parts.push(stream_info.trim().trim_matches('-').trim());
        }
        let mut parts = parts.iter();
        if let Some(part) = parts.next() {
          print!("{} => {}", group.name, part);
          for part in parts {
            print!(" - {}", part);
          }
          println!();
        }
      }
    }
  }
  if !found {
    if let Some(group_name) = group_name {
      return Err(ErrorKind::UnknownGroup(group_name.to_string()).into());
    }
    return Err("No groups found".into());
  }
  Ok(())
}
