use std::fs;
use std::io;

fn write_job(job: WriteJob) -> io::Result<()> {
    if let Some(parent) = job.dest_path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(&job.dest_path, &job.bytes);
}
