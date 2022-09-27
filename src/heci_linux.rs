use crate::heci::*;
use std::fs::File;
use std::io;
use std::io::prelude::*;

impl Heci {
    fn new(device: &str) -> Self {
        let fd = File::open(device);
        Self { device = fd }
    }
}
impl HeciOp for Heci {
}

