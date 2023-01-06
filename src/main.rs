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

    /*
    Comment `let process ...` out to have git2 work just fine.

    With it, you'll get lots of errors a la:

    ```
    error: Failed to retrieve list of SSH authentication methods: Error waiting on socket; class=Ssh (23); code=Auth (-16)
    error: SSH could not read data: Error waiting on socket; class=Ssh (23)
    error: Failed to open SSH channel: Error waiting on socket; class=Ssh (23)
    ```

    (They key error here is: "Error waiting on socket")

    I believe what's happening is this:

    async-process (Command::new()) works internally by dealing with SIGCHLD etc.
    for process reaping. In other words, _this_ process starts receiving a bunch
    of SIGCHLD signals.

    This causes the following `poll()` (or `select()`, depending on
    implementation)

    https://github.com/libssh2/libssh2/blob/master/src/session.c#L645

    ... to return with errno EINTR on each signal received:

    https://github.com/libssh2/libssh2/blob/master/src/session.c#L678

    ... which is the error that ultimately surfaces in git2-rs.

    The question then is: How does one even use git2-rs from within a process
    that receives signals periodically? If we simply mask it out, whomever
    depends on the signal gets broken. If not, we'll have to deal with the error
    all the way up here, even though we should really have been able to simply
    restart a simple poll on eg. a recv() on socket. Any help or thoughts
    appreciated!
    */

    let _process = thread::spawn(move || {
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
