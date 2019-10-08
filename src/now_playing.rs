use clap::{Arg, ArgMatches, App};
use ronor::{Sonos, PlaybackState};
use std::process::exit;
use super::Result;

pub fn build() -> App<'static, 'static> {
  App::new("now-playing").alias("np")
    .about("Describes what is currently playing")
    .arg(Arg::with_name("GROUP"))
}

pub fn run(sonos: &mut Sonos, matches: &ArgMatches) -> Result<()> {
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
    if matches.value_of("GROUP").is_some() && !found {
      println!("The specified group was not found");
      exit(1);
    }
    Ok(())
  })
}
