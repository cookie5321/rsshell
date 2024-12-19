use std::fs::File;
use std::io;
use std::io::{stdin, stdout, Write};
use std::os::fd::{AsRawFd, IntoRawFd};
use std::process::{Command, Stdio};
use nix::unistd::{close, dup, dup2, pipe};

fn main() -> io::Result<()> {
    // println!("######### oh-my-shell starts! #########");
    // loop {
    //     // main interaction loop starts
    //     print!(">>> ");
    //     stdout().flush().unwrap();
    //     let mut input = String::new();
    //     stdin().read_line(&mut input)?;
    //
    //     let mut child = Command::new("tee").arg("outt.txt").stdin(Stdio::inherit()).spawn()?;
    //
    //     // let mut child_stdin = child.stdin.unwrap().into_raw_fd();
    //     match input.find('>') {
    //         None => {}
    //         Some(x) => {}
    //     }
    //     let fd = File::open("awef.txt")?.as_raw_fd();
    //     nix::unistd::dup2(fd, 1)?;
    //     let status = child.wait();
    //     nix::unistd::close(fd)?;
    // }
    // Open the file you want to pass as stdin to the first child
    let file = File::open("awef.txt")?; // Replace with your file path
    let raw_fd = file.as_raw_fd(); // Get the raw file descriptor

    let saved_stdout = dup(1)?;

    // Create a pipe to connect the two processes
    let (read_fd, write_fd) = pipe().expect("Failed to create pipe");


    // Duplicate the file descriptor to stdin (fd 0) for the first child
    dup2(raw_fd, 0).expect("Failed to redirect stdin for child 1");

    // Duplicate the pipe's write end to stdout (fd 1) for the first child
    dup2(write_fd.as_raw_fd(), 1).expect("Failed to redirect stdout for child 1");

    // Spawn the first child process (cat) to read the file and write to the pipe
    let mut child1 = Command::new("grep") // Replace "cat" with your desired command
        .arg("asdf")
        .stdin(Stdio::inherit()) // stdin is inherited (we'll redirect manually)
        .stdout(Stdio::inherit()) // We'll redirect stdout manually
        .spawn()?;


    // Close the write end of the pipe in the parent (we've passed it to child1)
    // close(write_fd.as_raw_fd()).expect("Failed to close write end of the pipe");
    child1.wait()?;

    // Duplicate the pipe's read end to stdin (fd 0) for the second child (grep)
    let a = read_fd.into_raw_fd();
    dup2(a, 0).expect("Failed to redirect stdin for child 2");
    close(a).expect("Failed to close read end of the pipe");
    close(write_fd.into_raw_fd())?;
    dbg!(saved_stdout);
    dup2(saved_stdout, 1)?;
    // Spawn the second child process (grep) to read from the pipe
    let mut child2 = Command::new("cat") // Replace "grep" with your desired command
        // .arg("outtawefs.txt") // Replace with the pattern you want to search for
        .stdin(Stdio::inherit()) // stdin will be redirected manually
        .stdout(Stdio::inherit()) // stdout will be inherited (we want to see the output)
        .spawn()?;


    // Close the read end of the pipe in the parent (we've passed it to child2)

    // Wait for both child processes to finish
    let status1 = child1.wait()?;
    let status2 = child2.wait()?;

    Ok(())

}

// use std::fs::File;
// use std::os::unix::io::{AsRawFd, IntoRawFd};
// use std::process::{Command};
// use nix::unistd::{dup2, pipe, close};
//
// fn main() -> nix::Result<()> {
//     // Step 1: Create the first pipe for `echo` to `grep`
//     let (pipe_read, pipe_write) = pipe()?;
//
//     // Redirect stdout (1) to the write end of the pipe
//     dup2(pipe_write.as_raw_fd(), 1)?;
//     close(pipe_write.into_raw_fd())?; // Close the original pipe_write in the parent
//
//     // Spawn `echo "Hello, World!"` (inherits stdout redirection)
//     let mut echo_child = Command::new("echo")
//         .arg("Hello, World!")
//         .spawn()
//         .expect("Failed to spawn echo");
//
//     // Step 2: Redirect stdin (0) to the read end of the first pipe
//     dup2(pipe_read.as_raw_fd(), 0)?;
//     close(pipe_read.into_raw_fd())?; // Close the original pipe_read in the parent
//
//     // Create the second pipe for `grep` to `output.txt`
//     let (grep_read, grep_write) = pipe()?;
//
//     // Redirect stdout (1) to the write end of the second pipe
//     dup2(grep_write.as_raw_fd(), 1)?;
//     close(grep_write.into_raw_fd())?; // Close the original pipe_write in the parent
//
//     // Step 3: Redirect stdin (0) to the read end of the second pipe for `output.txt`
//     dup2(grep_read.as_raw_fd(), 0)?;
//     close(grep_read.into_raw_fd())?; // Close the original pipe_read in the parent
//
//     // Redirect stdout (1) to a file
//     let output_file = File::create("output.txt").expect("Failed to create output file");
//     let output_fd = output_file.into_raw_fd();
//     dup2(output_fd, 1)?;
//
//     // Spawn `grep Hello` (inherits stdin and stdout redirection)
//     let mut grep_child = Command::new("grep")
//         .arg("Hello")
//         .spawn()
//         .expect("Failed to spawn grep");
//
//
//     // Wait for child processes to finish
//     echo_child.wait().expect("Echo process failed");
//     grep_child.wait().expect("Grep process failed");
//
//     println!("Pipeline executed. Output written to 'output.txt'.");
//
//     Ok(())
// }
