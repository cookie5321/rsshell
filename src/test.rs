use nix::unistd::{dup2, close};
use std::fs::File;
use std::os::unix::io::AsRawFd;
use std::process::{Command, Stdio};

fn main() -> std::io::Result<()> {
    // Open the file you want to pass as stdin
    let file = File::open("example.txt")?; // Replace with your file path
    let raw_fd = file.as_raw_fd(); // Get the raw file descriptor

    // Prevent the file from being dropped too early
    // The `File` object needs to stay in scope to keep the file descriptor valid
    let _keep_file_alive = file;

    // Duplicate the file descriptor onto stdin (fd 0)
    dup2(raw_fd, 0).expect("Failed to duplicate file descriptor to stdin");

    // Spawn the child process
    let mut child = Command::new("cat") // Replace "cat" with your desired command
        .stdin(Stdio::inherit()) // Inherit stdin (now redirected)
        .stdout(Stdio::inherit()) // Inherit stdout for simplicity
        .spawn()?;

    // Wait for the child process to complete
    let status = child.wait()?;
    println!("Child process exited with status: {}", status);

    // No need to explicitly close `raw_fd`, as `_keep_file_alive` ensures the file descriptor is valid
    Ok(())
}
