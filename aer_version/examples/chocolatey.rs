// Copyright (c) 2021 Kim J. Nordmo and WormieCorp.
// Licensed under the MIT license. See LICENSE.txt file in the project

use aer_version::chocolatey::ChocoVersion;
use aer_version::{FixVersion, SemVersion};

fn main() {
    println!(
        "The following are examples of converting raw strings to chocolatey compatible versions:"
    );
    const VALUES: [&str; 11] = [
        "1",
        "2.0",
        "3.1.5",
        "2.2.1.0",
        "3.0-alpha",
        "2.5-beta.34",
        "1.5-ceta-50",
        "0.8-numero-uno-5",
        "1.0.0-alpha65",
        "2.2-55",
        "1.5.2.6-alpha.22+some-metadata",
    ];

    let mut chocos = vec![];

    for val in VALUES.iter() {
        let choco = ChocoVersion::parse(val).unwrap();
        println!("{:>20} => {}", val, choco);
        chocos.push(choco);
    }

    println!("And finally an example of creating a fix version");

    let mut choco = ChocoVersion::new(4, 2);
    choco.add_fix().unwrap();
    println!("\n{:>20} => {}", "4.2", choco);
    chocos.push(choco);

    let mut choco = ChocoVersion::parse("1.0-alpha").unwrap();
    choco.add_fix().unwrap();
    println!("{:>20} => {}", "1.0-alpha", choco);
    chocos.push(choco);

    println!("And then converting all of the choco versions back to semver!");

    for choco in chocos {
        let choco_clone = choco.clone();
        let semver: SemVersion = choco.into();
        println!("{:>20} => {}", choco_clone, semver);
    }
}
