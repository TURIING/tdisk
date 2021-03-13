#![feature(async_closure)]
#![feature(once_cell)]

mod util;
mod object;
mod error;
mod option;
mod variable;

#[macro_use]
extern crate serde;
use structopt::StructOpt;


use error::{Result, Error};
use option::{AppArgs, Sub};
use crate::variable::INFO;

#[tokio::main]
async fn main() -> Result<()>{
    util::init()?;

    let app_args = AppArgs::from_args();
    match app_args.up {
        Some(sub) => match sub {
            Sub::Down{Src: src, Dec: dec} => {
                match dec {
                    Some(p) => object::get(src, Some(p)).await,
                    None => object::get(src, None).await,
                }
            },
            Sub::Up{Src: src, Dec:dec} => {
                object::upload(src, dec).await;
            }

        },
        None => println!("Missing parameter"),
    };


    Ok(())
}

