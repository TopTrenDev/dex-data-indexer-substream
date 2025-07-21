#![allow(dead_code)]
#![allow(unused_variables)]
#![allow(non_snake_case)]

mod dapps;
mod pb;
mod utils;
mod trade_instruction;

use pb::sf::solana::dex::trades::v1::{ Output, TradeData };
use substreams::log;
use substreams_solana::pb::sf::solana::r#type::v1::InnerInstructions;
use substreams_solana::pb::sf::solana::r#type::v1::{ Block, Transaction };
use utils::get_mint;
use utils::{ convert_to_date, get_amt };
