// MIT License
//
// Copyright (c) 2017 Toshiki Teramura
//
// Permission is hereby granted, free of charge, to any person obtaining a copy
// of this software and associated documentation files (the "Software"), to deal
// in the Software without restriction, including without limitation the rights
// to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
// copies of the Software, and to permit persons to whom the Software is
// furnished to do so, subject to the following conditions:
//
// The above copyright notice and this permission notice shall be included in all
// copies or substantial portions of the Software.
//
// THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
// IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
// FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
// AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
// LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
// OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
// SOFTWARE.

extern crate crypto;
extern crate curl;
extern crate bzip2;
extern crate tar;

use crypto::md5;
use crypto::digest::Digest;

use curl::easy::Easy;
use bzip2::read::BzDecoder;
use tar::Archive;

use std::env::var;
use std::path::*;
use std::fs::{self, File};
use std::io::*;


// Use `conda search --json --platform 'win-64' mkl-static`
// to query the metadata of conda package (includes MD5 sum).

#[cfg(target_os = "linux")]
mod mkl {
    pub const LIB_PATH: &'static str = "lib";

    pub const DLS: &[(&'static str, &'static str, &'static str)] = &[
        ("mkl-static-2019.1-intel_144.tar.bz2", 
         "https://conda.anaconda.org/intel/linux-64/mkl-static-2019.1-intel_144.tar.bz2",
         "37e3a60ff2643cf40b5cf9d2c183588c")
    ];
}

#[cfg(target_os = "macos")]
mod mkl {
    pub const LIB_PATH: &'static str = "lib";

    pub const DLS: &[(&'static str, &'static str, &'static str)] = &[
        ("mkl-static-2019.1-intel_144.tar.bz2", 
         "https://conda.anaconda.org/intel/osx-64/mkl-static-2019.1-intel_144.tar.bz2", 
         "74a186a5e325146c7de7e1e1c8fc3bc3")
    ];
}

#[cfg(target_os = "windows")]
mod mkl {
    pub const LIB_PATH: &'static str = "lib";

    pub const DLS: &[(&'static str, &'static str, &'static str)] = &[
        ("mkl-static-2019.1-intel_144.tar.bz2", 
         "https://conda.anaconda.org/intel/win-64/mkl-static-2019.1-intel_144.tar.bz2", 
         "0b65a55b6bcda83392e9defff8e1edbe")
    ];
}

fn download(uri: &str, filename: &str, out_dir: &Path) {

    let out = PathBuf::from(out_dir.join(filename));

    // Download the tarball.
    let f = File::create(&out).unwrap();
    let mut writer = BufWriter::new(f);
    let mut easy = Easy::new();
    easy.follow_location(true).unwrap();
    easy.url(&uri).unwrap();
    easy.write_function(move |data| {
        Ok(writer.write(data).unwrap())
    }).unwrap();
    easy.perform().unwrap();

    let response_code = easy.response_code().unwrap();
    if response_code != 200 {
        panic!("Unexpected response code {} for {}", response_code, uri);
    }
}

fn calc_md5(path: &Path) -> String {
    let mut sum = md5::Md5::new();
    let mut f = BufReader::new(fs::File::open(path).unwrap());
    let mut buf = Vec::new();
    f.read_to_end(&mut buf).unwrap();
    sum.input(&buf);
    sum.result_str()
}

fn extract<P: AsRef<Path>, P2: AsRef<Path>>(archive_path: P, extract_to: P2) {
    let file = File::open(archive_path).unwrap();
    let unzipped = BzDecoder::new(file);
    let mut a = Archive::new(unzipped);
    a.unpack(extract_to).unwrap();
}

fn main() {
    let out_dir = PathBuf::from(var("OUT_DIR").unwrap());

    for (archive, uri, md5) in mkl::DLS {
        let archive_path = out_dir.join(archive);
        if archive_path.exists() && calc_md5(&archive_path) == *md5 {
            println!("Use existings archive");
        } else {
            println!("Download archive");
            download(uri, archive, &out_dir);
            extract(&archive_path, &out_dir);
            
            let sum = calc_md5(&archive_path);
            if sum != *md5 {
                panic!(
                    "check sum of downloaded archive is incorrect: md5sum={}",
                    sum
                );
            }
        }
    }
    
    println!("cargo:rustc-link-search={}", out_dir.join(mkl::LIB_PATH).display());
    println!("cargo:rustc-link-lib=static=mkl_intel_lp64");
    println!("cargo:rustc-link-lib=static=mkl_sequential");
    println!("cargo:rustc-link-lib=static=mkl_core");
}