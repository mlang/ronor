use clap::{Arg, ArgMatches, App};
use ronor::Sonos;
use std::process::exit;
use super::Result;

pub const NAME: &'static str = "get-group-volume";

pub fn build() -> App<'static, 'static> {
  App::new(NAME)
    .about("Get group volume")
    .arg(Arg::with_name("GROUP"))
}

pub fn run(sonos: &mut Sonos, matches: &ArgMatches) -> Result<()> {
  with_authorization!(sonos, {
    let mut found = false;
    let group_name = matches.value_of("GROUP");
    for household in sonos.get_households()?.iter() {
      for group in sonos.get_groups(&household)?.groups.iter().filter(|group|
        group_name.map_or(true, |name| name == group.name)
      ) {
        found = true;
        let group_volume = sonos.get_group_volume(&group)?;
        println!("{:?} => {:#?}", group.name, group_volume);
      }
    }
    if group_name.is_some() && !found {
      println!("The specified group was not found");
      exit(1);
    }
    Ok(())
  })
}
