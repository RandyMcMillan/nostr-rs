use offline::offline;
mod offline;
use online::online;
mod online;
use paid::paid;
mod paid;

use nip0::nip0;
mod nip0;
use nip1::nip1;
mod nip1;
use nip2::nip2;
mod nip2;
use nip3::nip3;
mod nip3;
use nip4::nip4;
mod nip4;
use nip5::nip5;
mod nip5;
use nip34::nip34;
mod nip34;


pub fn run(name: &str) {
    match name {
        "offline" => offline(),
        "online" => online(),
        "paid" => paid(),
        "nip0" => nip0(),
        "nip1" => nip1(),
        "nip2" => nip2(),
        "nip3" => nip3(),
        "nip4" => nip4(),
        "nip5" => nip5(),
        "nip34" => nip34(),
        _ => panic!("Invalid option"),
    };
}
