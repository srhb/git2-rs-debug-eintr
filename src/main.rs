/*
 * libgit2 "clone" example
 *
 * Written by the libgit2 contributors
 *
 * To the extent possible under law, the author(s) have dedicated all copyright
 * and related and neighboring rights to this software to the public domain
 * worldwide. This software is distributed without any warranty.
 *
 * You should have received a copy of the CC0 Public Domain Dedication along
 * with this software. If not, see
 * <http://creativecommons.org/publicdomain/zero/1.0/>.
 */


use std::{env, time};
use std::path::Path;
use std::thread;

use async_process::{Command, Stdio};

use git2::build::{CheckoutBuilder, RepoBuilder};
use git2::{FetchOptions, RemoteCallbacks, Cred};

use structopt::StructOpt;
use tempdir::TempDir;

#[derive(StructOpt)]
struct Args {
    #[structopt(name = "url")]
    arg_url: String,
}

fn run(args: &Args, dir: &Path) -> Result<(), git2::Error> {
    let mut cb = RemoteCallbacks::new();
    cb.credentials(|_url, username_from_url, _allowed_types| {
        Cred::ssh_key(
            username_from_url.unwrap(),
            None,
            std::path::Path::new(&format!("{}/.ssh/id_rsa", env::var("HOME").unwrap())),
            None,
        )
    });

    let co = CheckoutBuilder::new();

    let mut fo = FetchOptions::new();
    fo.remote_callbacks(cb);
    RepoBuilder::new()
        .fetch_options(fo)
        .with_checkout(co)
        .clone(&args.arg_url, dir)?;

    Ok(())
}

fn main() {
    let args = Args::from_args();

    let process = thread::spawn(move || {
        loop {
            Command::new("true")
                .stdout(Stdio::piped())
                .spawn().expect("Failed to spawn true");
        }
    });

    let git = thread::spawn(move || {
        for _i in 1..100 {
            let tmp_dir = TempDir::new("git2-rs-eintr-clone").expect("Could not create tmp_dir");
            match run(&args, tmp_dir.path()) {
                Ok(()) => println!("Cloned succesfully!"),
                Err(e) => println!("error: {}", e),
            }
            tmp_dir.close().expect("Woops, couldn't get rid of dir");
        }
    });

    git.join().expect("Wanted to hang forever, failed");
}
