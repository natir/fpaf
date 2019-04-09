/*
Copyright (c) 2018 Pierre Marijon <pierre.marijon@inria.fr>

Permission is hereby granted, free of charge, to any person obtaining a copy
of this software and associated documentation files (the "Software"), to deal
in the Software without restriction, including without limitation the rights
to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
copies of the Software, and to permit persons to whom the Software is
furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in all
copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
SOFTWARE.
*/

extern crate bzip2;
extern crate csv;
extern crate clap;
extern crate flate2;
extern crate petgraph;
extern crate regex;
extern crate serde;
extern crate xz2;

#[macro_use]
extern crate serde_derive;

#[macro_use]
extern crate enum_primitive;

#[cfg(test)]
#[macro_use]
extern crate lazy_static;

/* project mod */
mod io;
mod cli;
mod file;
//mod work;
mod filter;
mod generator;

use cli::Filters;
use io::MappingRecord;

fn main() {

    let matches = cli::parser();

    /* Manage input and output file */
    let compression: file::CompressionFormat;
    let input: Box<std::io::Read>;
    let (input, compression) = file::get_input(matches.value_of("input").unwrap());

    let format = if matches.is_present("format") {
        match matches.value_of("format").unwrap() {
            "paf" => io::MappingFormat::Paf,
            "mhap" => io::MappingFormat::Mhap,
            _ => io::MappingFormat::Paf,
        }
    } else {
        io::MappingFormat::Paf
    };

    let out_compression = file::choose_compression(
        compression,
        matches.is_present("compression-out"),
        matches.value_of("compression-out").unwrap_or("no"),
    );

    let mut output: std::io::BufWriter<Box<std::io::Write>> =
        std::io::BufWriter::new(file::get_output(matches.value_of("output").unwrap(), out_compression));

    let mut writer = io::paf::Writer::new(output);
    let mut reader = io::paf::Reader::new(input);
    let drop = cli::Drop::new(&matches);
    let keep = cli::Keep::new(&matches);
    let mut modifier = cli::Modifier::new(&matches);

    let mut position = 0;
    for result in reader.records() {
        let mut record = result.expect("Trouble during read of input mapping");

        // keep
        if !keep.pass(&record) {
            continue
        }

        // drop
        if !drop.pass(&record) {
            continue
        }

        writer.write(&record)
            .expect("Trouble during write of output");

        let new_position = output.seek(std::io::SeekFrom::Current(0));
        record.set_position((position, new_position));
        
        // modifier
        modifier.pass(&mut record);

        position = new_position;
    }

    // close modifier
    modifier.write();
}
