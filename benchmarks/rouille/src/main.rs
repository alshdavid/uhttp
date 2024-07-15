// Copyright (c) 2016 The Rouille developers
// Licensed under the Apache License, Version 2.0
// <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT
// license <LICENSE-MIT or http://opensource.org/licenses/MIT>,
// at your option. All files in the project carrying such
// notice may not be copied, modified, or distributed except
// according to those terms.

#![allow(unreachable_code)]
#[macro_use]
extern crate rouille;

fn main() {
    println!("Now listening on localhost:8000");

    // The `start_server` starts listening forever on the given address.
    rouille::start_server("localhost:8080", move |request| {
      rouille::Response::text("hello world")

    });
}