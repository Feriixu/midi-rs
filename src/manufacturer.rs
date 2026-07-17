// Copyright 2015 Sam Doshi (sam@metal-fish.co.uk)
//
// Licensed under the MIT License <LICENSE or http://opensource.org/licenses/MIT>.
// This file may not be copied, modified, or distributed except according to those terms.

use heapless::Vec;

use types::U7;
use utils::mask7;

#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub enum Manufacturer {
    OneByte(U7),
    ThreeByte(U7, U7, U7)
}

impl Manufacturer {
    pub fn to_u7s(&self) -> Vec<U7, 3> {
        match self {
            &Manufacturer::OneByte(b) => Vec::from_slice(&[mask7(b)]).unwrap(),
            &Manufacturer::ThreeByte(b1, b2, b3) => {
                Vec::from_slice(&[mask7(b1), mask7(b2), mask7(b3)]).unwrap()
            }
        }
    }
}
