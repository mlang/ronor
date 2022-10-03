use crate::Result;
use clap::{Command, Arg, ArgMatches};
use ronor::{Capability, HouseholdId, Player, PlayerId, Sonos};

pub const NAME: &str = "inventory";

pub fn build() -> Command {
  Command::new(NAME)
    .about("Describes available households, groups and logical players")
    .arg(
      Arg::new("AUDIO_CLIP")
        .short('c')
        .long("audio-clip")
        .help("Limits to players with the audio-clip capability")
    )
    .arg(
      Arg::new("HT_PLAYBACK")
        .short('t')
        .long("ht-playback")
        .help("Limits to players with the home theater playback capability")
    )
    .arg(
      Arg::new("LINE_IN")
        .short('l')
        .long("line-in")
        .help("Only show players with the line-in capability")
    )
    .arg(
      Arg::new("PLAYERS")
        .long("players")
        .help("Only show players")
    )
    .arg(
      Arg::new("HOUSEHOLD")
        .long("household-id")
        .num_args(1)
        .value_name("IDENTIFIER")
        .help("Limits output to a specific household")
    )
}

pub fn run(sonos: &mut Sonos, matches: &ArgMatches) -> Result<()> {
  let household_id = matches
    .get_one::<String>("HOUSEHOLD")
    .map(|id| HouseholdId::new(id.to_string()));
  let audio_clip = if matches.contains_id("AUDIO_CLIP") {
    Some(Capability::AudioClip)
  } else {
    None
  };
  let ht_playback = if matches.contains_id("HT_PLAYBACK") {
    Some(Capability::HtPlayback)
  } else {
    None
  };
  let line_in = if matches.contains_id("LINE_IN") {
    Some(Capability::LineIn)
  } else {
    None
  };
  for household in sonos.get_households()?.iter().filter(|household| {
    household_id
      .as_ref()
      .map_or(true, |household_id| household_id == &household.id)
  }) {
    if household_id.is_none() {
      println!("Household: {}", household.id);
    }
    let targets = sonos.get_groups(&household)?;
    fn find_player<'a>(
      players: &'a [Player],
      player_id: &PlayerId
    ) -> Option<&'a Player> {
      players.iter().find(|player| &player.id == player_id)
    }
    if matches.contains_id("PLAYERS")
      || audio_clip.is_some()
      || ht_playback.is_some()
      || line_in.is_some()
    {
      for player in targets
        .players
        .iter()
        .filter(|player| {
          audio_clip
            .as_ref()
            .map_or(true, |capability| player.capabilities.contains(&capability))
        })
        .filter(|player| {
          ht_playback
            .as_ref()
            .map_or(true, |capability| player.capabilities.contains(&capability))
        })
        .filter(|player| {
          line_in
            .as_ref()
            .map_or(true, |capability| player.capabilities.contains(&capability))
        })
      {
        println!("{}", player.name);
      }
    } else {
      for group in targets.groups.iter() {
        print!("{}", group.name);
        let mut player_ids = group.player_ids.iter();
        if let Some(player_id) = player_ids.next() {
          if let Some(player) = find_player(&targets.players, player_id) {
            print!(" = {}", player.name);
            for player_id in player_ids {
              if let Some(player) = find_player(&targets.players, player_id) {
                print!(" + {}", player.name);
              }
            }
          }
        }
        println!();
      }
    }
  }
  Ok(())
}
