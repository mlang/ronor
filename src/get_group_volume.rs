use clap::{Arg, ArgMatches, App};
use ronor::Sonos;
use std::process::exit;
use super::Result;

pub fn build() -> App<'static, 'static> {
  App::new("get-group-volume")
    .about("Get group volume")
    .arg(Arg::with_name("GROUP"))
}

pub fn run(sonos: &mut Sonos, matches: &ArgMatches) -> Result<()> {
  with_authorization!(sonos, {
    let mut found = false;
    for household in sonos.get_households()?.iter() {
      for group in sonos.get_groups(&household)?.groups.iter() {
        if matches.value_of("GROUP").map_or(true, |name| name == group.name) {
          found = true;
          let group_volume = sonos.get_group_volume(&group)?;
          println!("{:?} => {:#?}", group.name, group_volume);
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

