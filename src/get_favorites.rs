use clap::{Arg, ArgMatches, App};
use ronor::Sonos;
use super::Result;

pub fn build() -> App<'static, 'static> {
  App::new("get-favorites")
    .about("Get list of favorites")
}

pub fn run(sonos: &mut Sonos, _matches: &ArgMatches) -> Result<()> {
  with_authorization!(sonos, {
    for household in sonos.get_households()?.iter() {
      for favorite in sonos.get_favorites(&household)?.items.iter() {
        println!("{}", favorite.name);
      }
    }
    Ok(())
  })
}

