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

#[cfg(target_os = "linux")]
mod mkl {
    pub const ARCHIVE: &'static str = "mkl_linux.tar.xz";
    pub const MD5SUM: &'static str = "9c6c0d8609ffb94af492d0c07fb7f22c";
    pub const URI: &'static str = "https://anaconda.org/anaconda/mkl/2019.1/download/linux-64/mkl-2019.1-144.tar.bz2";
    pub const SO_PATH: &'static str = "lib";
}

#[cfg(target_os = "macos")]
mod mkl {
    pub const ARCHIVE: &'static str = "mkl_osx.tar.xz";
    pub const MD5SUM: &'static str = "c9598d7fb664a16f0abfc2cb71491b62";
    pub const URI: &'static str = "https://anaconda.org/anaconda/mkl/2019.1/download/osx-64/mkl-2019.1-144.tar.bz2";
    pub const SO_PATH: &'static str = "lib";
}

#[cfg(target_os = "windows")]
mod mkl {
    pub const ARCHIVE: &'static str = "mkl_win.tar.xz";
    pub const MD5SUM: &'static str = "429418273137ee591a55a0a210cf7436";
    pub const URI: &'static str = "https://anaconda.org/anaconda/mkl/2019.1/download/win-64/mkl-2019.1-144.tar.bz2";
    pub const SO_PATH: &'static str = "Library/bin";
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
    let archive_path = out_dir.join(mkl::ARCHIVE);

    if archive_path.exists() && calc_md5(&archive_path) == mkl::MD5SUM {
        println!("Use existings archive");
    } else {
        println!("Download archive");
        download(mkl::URI, mkl::ARCHIVE, &out_dir);
        extract(&archive_path, &out_dir);
        let sum = calc_md5(&archive_path);
        if sum != mkl::MD5SUM {
            panic!(
                "check sum of downloaded archive is incorrect: md5sum={}",
                sum
            );
        }
    }
    
    println!("cargo:rustc-link-search={}", out_dir.join(mkl::SO_PATH).display());
    println!("cargo:rustc-link-lib=mkl_intel_lp64");
    println!("cargo:rustc-link-lib=mkl_sequential");
    println!("cargo:rustc-link-lib=mkl_core");
    println!("cargo:rustc-link-lib=m");
}