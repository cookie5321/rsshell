// use std::ffi::CString;
// use std::fs::File;
// use std::{io, process};
// use std::io::{stdin, stdout, Write};
// use std::os::fd::{AsRawFd, IntoRawFd};
// use std::process::{Command, Stdio};
// use nix::sys::wait::wait;
// use nix::unistd::{fork, ForkResult, execvp};
// use nix::unistd::{close, dup, dup2, pipe};
//
// fn main() -> io::Result<()> {
//     // println!("######### oh-my-shell starts! #########");
//     // loop {
//     //     // main interaction loop starts
//     //     print!(">>> ");
//     //     stdout().flush().unwrap();
//     //     let mut input = String::new();
//     //     stdin().read_line(&mut input)?;
//     //
//     //     let mut child = Command::new("tee").arg("outt.txt").stdin(Stdio::inherit()).spawn()?;
//     //
//     //     // let mut child_stdin = child.stdin.unwrap().into_raw_fd();
//     //     match input.find('>') {
//     //         None => {}
//     //         Some(x) => {}
//     //     }
//     //     let fd = File::open("awef.txt")?.as_raw_fd();
//     //     nix::unistd::dup2(fd, 1)?;
//     //     let status = child.wait();
//     //     nix::unistd::close(fd)?;
//     // }
//     // Open the file you want to pass as stdin to the first child
//     let file = File::open("awef.txt")?; // Replace with your file path
//     let raw_fd = file.as_raw_fd(); // Get the raw file descriptor
//
//     let saved_stdout = dup(1)?;
//
//     // Create a pipe to connect the two processes
//     let (read_fd, write_fd) = pipe().expect("Failed to create pipe");
//
//
//     match unsafe { fork() } {
//         Ok(ForkResult::Parent { child }) => {}
//         Ok(ForkResult::Child) => {
//             let program = CString::new("cat")?;
//             let args = [program.clone()];
//
//             // Duplicate the file descriptor to stdin (fd 0) for the first child
//             dup2(raw_fd, 0).expect("Failed to redirect stdin for child 1");
//
//             // Duplicate the pipe's write end to stdout (fd 1) for the first child
//             dup2(write_fd.as_raw_fd(), 1).expect("Failed to redirect stdout for child 1");
//             print!("B");
//
//             match unsafe { fork() } {
//                 Ok(ForkResult::Parent { child }) => {}
//                 Err(_) => {}
//                 Ok(ForkResult::Child) => {
//                     print!("a");
//                     let program = CString::new("grep")?;
//                     let args = [program.clone(), CString::new("asdf")?];
//                     // Duplicate the file descriptor to stdin (fd 0) for the first child
//                     dup2(read_fd.as_raw_fd(), 0).expect("Failed to redirect stdin for child 1");
//
//                     // Duplicate the pipe's write end to stdout (fd 1) for the first child
//                     dup2(saved_stdout, 1).expect("Failed to redirect stdout for child 1");;
//                     execvp(&program, &args)?;
//                 }
//             }
//             execvp(&program, &args)?;
//         }
//         Err(_) => { panic!("fork error"); }
//     }
//
//     // // Spawn the first child process (cat) to read the file and write to the pipe
//     // let mut child1 = Command::new("grep") // Replace "cat" with your desired command
//     //     .arg("asdf")
//     //     .stdin(Stdio::inherit()) // stdin is inherited (we'll redirect manually)
//     //     .stdout(Stdio::inherit()) // We'll redirect stdout manually
//     //     .spawn()?;
//
//
//     // Close the write end of the pipe in the parent (we've passed it to child1)
//     // close(write_fd.as_raw_fd()).expect("Failed to close write end of the pipe");
//     // child1.wait()?;
//
//
//     // Duplicate the pipe's read end to stdin (fd 0) for the second child (grep)
//     dup2(read_fd.as_raw_fd(), 0).expect("Failed to redirect stdin for child 2");
//     dbg!(&saved_stdout);
//
//     dup2(saved_stdout, 1)?;
//     // close(a).expect("Failed to close read end of the pipe");
//     // close(write_fd.into_raw_fd())?;
//     // dbg!(saved_stdout);
//     // dup2(saved_stdout, 1)?;
//     // Spawn the second child process (grep) to read from the pipe
//     // let mut child2 = Command::new("cat") // Replace "grep" with your desired command
//     //     // .arg("outtawefs.txt") // Replace with the pattern you want to search for
//     //     .stdin(Stdio::inherit()) // stdin will be redirected manually
//     //     .stdout(Stdio::inherit()) // stdout will be inherited (we want to see the output)
//     //     .spawn()?;
//
//
//     match unsafe { fork() } {
//         Ok(ForkResult::Parent { child }) => {}
//         Ok(ForkResult::Child) => {
//             let program = CString::new("cat")?;
//             let args = [program.clone()];
//
//
//             execvp(&program, &args)?;
//             process::exit(0);
//         }
//         Err(_) => { panic!("fork error"); }
//     }
//
//     // Close the read end of the pipe in the parent (we've passed it to child2)
//
//     // Wait for both child processes to finish
//     // let status1 = child1.wait()?;
//     // let status2 = child2.wait()?;
//
//     Ok(())
//
// }

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
use nix::fcntl::{open, OFlag};
use nix::sys::stat::Mode;
use nix::unistd::{close, dup2, execvp, fork, pipe, ForkResult};
use std::ffi::CString;
use std::os::unix::io::RawFd;
use std::process::exit;
use std::io::{self, Write};
use std::os::fd::{AsRawFd, IntoRawFd};

fn execute_command(commands: Vec<Vec<String>>) {
    let mut prev_pipe: Option<(RawFd, RawFd)> = None;

    let mut final_output_fd: Option<RawFd> = None; // To track output redirection

    let mut children = Vec::new(); // To track child PIDs

    for (i, command) in commands.iter().enumerate() {
        // Check for output redirection in the last command
        if i == commands.len() - 1 {
            if let Some(pos) = command.iter().position(|x| x == ">") {
                final_output_fd = match open(command[pos + 1].as_str(), OFlag::O_WRONLY | OFlag::O_CREAT | OFlag::O_TRUNC, Mode::S_IRWXU) {
                    Ok(fd) => Some(fd),
                    Err(err) => {
                        eprintln!("Failed to open file for output redirection: {}", err);
                        exit(1);
                    }
                };
            }
        }


        // Create a pipe for the next process, if necessary
        let next_pipe = if i < commands.len() - 1 {
            match pipe() {
                Ok(p) => Some(p),
                Err(err) => {
                    eprintln!("Failed to create pipe: {}", err);
                    exit(1);
                }
            }
        } else {
            None
        };

        match unsafe {fork()} {
            Ok(ForkResult::Child) => {
                // Handle input redirection if applicable
                if i == 0 {
                    if let Some(pos) = command.iter().position(|x| x == "<") {
                        let file_fd = match open(command[pos + 1].as_str(), OFlag::O_RDONLY, Mode::empty()) {
                            Ok(fd) => fd,
                            Err(err) => {
                                eprintln!("Failed to open file for input redirection: {}", err);
                                exit(1);
                            }
                        };
                        if let Err(err) = dup2(file_fd, 0) {
                            eprintln!("Failed to redirect stdin: {}", err);
                            exit(1);
                        }
                        let _ = close(file_fd);
                    }
                }


                // Redirect input from the previous pipe, if applicable
                if let Some((read_fd, _)) = prev_pipe {
                    if let Err(err) = dup2(read_fd, 0) {
                        eprintln!("Failed to redirect stdin from pipe: {}", err);
                        exit(1);
                    }
                }

                // Redirect output to the next pipe, if applicable
                if let Some((_, ref write_fd)) = next_pipe {
                    if let Err(err) = dup2(write_fd.as_raw_fd(), 1) {
                        eprintln!("Failed to redirect stdout to pipe: {}", err);
                        exit(1);
                    }
                }
                if i == commands.len() - 1 {
                    if let Some(output_fd) = final_output_fd {
                        if let Err(err) = dup2(output_fd, 1) {
                            eprintln!("Failed to redirect stdout to file: {}", err);
                            exit(1);
                        }
                        let _ = close(output_fd);
                    }
                }
                // Close unused file descriptors
                if let Some((read_fd, write_fd)) = prev_pipe {
                    let _ = close(read_fd);
                    let _ = close(write_fd);
                }
                if let Some((read_fd, write_fd)) = next_pipe {
                    let _ = close(read_fd.into_raw_fd());
                    let _ = close(write_fd.into_raw_fd());
                }

                // Execute the command
                let args: Vec<CString> = command
                    .iter()
                    .filter(|x| x != &"<" && x != &">")
                    .map(|arg| CString::new(arg.as_str()).unwrap())
                    .collect();
                if let Err(err) = execvp(&args[0], &args) {
                    eprintln!("Failed to execute command: {}", err);
                    exit(1);
                }
            }
            Ok(ForkResult::Parent{child}) => {

                // Track child processes
                children.push(nix::unistd::getpid());

                // Close unused file descriptors in the parent
                if let Some((read_fd, write_fd)) = prev_pipe {
                    let _ = close(read_fd);
                    let _ = close(write_fd);
                }
                prev_pipe = next_pipe.map( |(read_fd, write_fd)| (read_fd.into_raw_fd(), write_fd.into_raw_fd()));
            }
            Err(err) => {
                eprintln!("Failed to fork: {}", err);
                exit(1);
            }
        }
    }

    // Close any remaining file descriptors in the parent
    if let Some((read_fd, write_fd)) = prev_pipe {
        let _ = close(read_fd);
        let _ = close(write_fd);
    }

    // Wait for all children to finish
    for _ in children {
        let _ = nix::sys::wait::wait();
    }
}

fn main() {
    loop {
        print!("shell> ");
        io::stdout().flush().unwrap();

        // Read user input
        let mut input = String::new();
        if io::stdin().read_line(&mut input).is_err() {
            eprintln!("Failed to read input");
            continue;
        }

        let input = input.trim();
        if input.is_empty() {
            continue;
        }
        if input == "exit" {
            break;
        }

        // Parse the input into commands and arguments
        let mut commands: Vec<Vec<String>> = Vec::new();
        let mut current_command: Vec<String> = Vec::new();
        for token in input.split_whitespace() {
            if token == "|" {
                commands.push(current_command);
                current_command = Vec::new();
            } else {
                current_command.push(token.to_string());
            }
        }
        commands.push(current_command);

        // Execute the parsed commands
        execute_command(commands);
    }
}
