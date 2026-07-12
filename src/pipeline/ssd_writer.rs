use std::fs;
use std::io;

use crate::pipeline::WriteJob;
use crossbeam_channel::Receiver;

fn write_job(job: &WriteJob) -> io::Result<()> {
    if let Some(parent) = job.dest_path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(&job.dest_path, &job.bytes)
}

pub fn run(rx: Receiver<WriteJob>) {
    for job in rx {
        if let Err(e) = write_job(&job) {
            eprintln!("Error writing {}: {e}", job.dest_path.display());
        }
    }
}
