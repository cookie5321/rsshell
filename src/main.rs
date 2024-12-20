use nix::fcntl::{open, OFlag};
use nix::sys::stat::Mode;
use nix::unistd::{close, dup2, execvp, fork, pipe, ForkResult};
use std::ffi::CString;
use std::fs::File;
use std::os::unix::io::RawFd;
use std::process::exit;
use std::io::{self, Write};
use std::os::fd::{AsRawFd, IntoRawFd};
use nix::sys::wait::WaitStatus;

fn execute_command(mut commands: Vec<String>) -> nix::Result<()> {
    let mut prev_pipe: Option<(RawFd, RawFd)> = None;
    let mut output_fd: Option<RawFd> = None;
    let mut input_fd: Option<RawFd> = None;
    let mut children = Vec::new(); // To track child PIDs

    if let Some((command, file)) = commands[0].split_once(&String::from("<")) {
        input_fd = Some(File::open(file.trim()).expect("file open failed").into_raw_fd());
        commands[0] = command.trim().to_string();
    }

    if let Some((command, file)) = commands.last().unwrap().split_once(&String::from(">")) {
        output_fd = Some(File::create(file.trim()).expect("file open failed").into_raw_fd());
        let len = commands.len();
        commands[len - 1] = command.trim().to_string();
    }

    // dbg!(&commands);

    for (i, command) in commands.iter().enumerate() {

        // Create a pipe for the next process, if necessary
        let next_pipe = if i < commands.len() - 1 {
            match pipe() {
                Ok((x, y)) => Some((x.into_raw_fd(), y.into_raw_fd())),
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
                    if let Some(file_fd) = input_fd {
                        dup2(file_fd, 0).expect("L");
                        close(file_fd).expect("M");
                    }
                }

                // Redirect input from the previous pipe, if applicable
                if let Some((read_fd, write_fd)) = prev_pipe {
                    dup2(read_fd, 0)?;
                    // close(write_fd).expect("N");
                    // dbg!(read_fd);
                }

                // Redirect output to the next pipe, if applicable
                if let Some((_, write_fd)) = next_pipe {
                    dup2(write_fd, 1).expect("A");
                    // dbg!(write_fd);
                }

                if i == commands.len() - 1 {
                    if let Some(output_fd) = output_fd {
                        dup2(output_fd, 1).expect("B");
                        // dbg!(output_fd);
                        close(output_fd).expect("K");
                    }
                }
                // Close unused file descriptors
                if let Some((read_fd, write_fd)) = prev_pipe {
                    // let _ = close(read_fd);
                    // close(write_fd).expect("J");
                }
                if let Some((read_fd, write_fd)) = next_pipe {
                    // let _ = close(read_fd);
                    // let _ = close(write_fd);
                }

                // Execute the command
                let args: Vec<CString> = process_args(command);
                // dbg!(&args);
                if let Err(err) = execvp(&args[0], &args) {
                    eprintln!("Failed to execute command: {}", err);
                    exit(1);
                }
            }
            Ok(ForkResult::Parent{child}) => {

                // Track child processes
                // children.push(child);

                // dbg!("wait started");
                children.push(nix::sys::wait::waitpid(child, None).expect("C"));

                if let Some((a, b)) = next_pipe {
                    // dbg!(a, "closing");
                    close(b).expect("E");
                }

                // dbg!("wait ended");


                // Close unused file descriptors in the parent
                if let Some((read_fd, write_fd)) = prev_pipe {
                    close(read_fd).expect("H");
                    // close(write_fd).expect("I");
                }
                prev_pipe = next_pipe;
            }
            Err(err) => {
                eprintln!("Failed to fork: {}", err);
                exit(1);
            }
        }
    }

    if let Some(a) = output_fd {
        // dbg!(a, "closing");
        close(a).expect("D");
    }
    // Close any remaining file descriptors in the parent
    if let Some((read_fd, write_fd)) = prev_pipe {
        let _ = close(read_fd).expect("F");
        let _ = close(write_fd).expect("G");
    }

    // Wait for all children to finish
    for child in children {
        println!("[oh-my-shell] Child process terminated: pid {:}, {}",
            child.pid().unwrap(),
            match child {
                WaitStatus::Exited(_, b) => format!("status: {b}"),
                WaitStatus::Signaled(_, b, _) => format!("killed by the signal {:?}", b),
                _ => "unknown exit cause".to_string()
            });
    }

    Ok(())
}


fn process_args(input: &str) -> Vec<CString> {
    let mut args = Vec::new();
    let mut current_arg = String::new();
    let mut in_quotes = false;

    for c in input.chars() {
        if c == '"' {
            in_quotes = !in_quotes;
        } else if c == ' ' && !in_quotes {
            if !current_arg.is_empty() {
                args.push(CString::new(current_arg.clone()).unwrap());
                current_arg.clear();
            }
        } else {
            current_arg.push(c);
        }
    }

    if !current_arg.is_empty() {
        args.push(CString::new(current_arg).unwrap());
    }

    args
}

fn main() {
    print!("######## oh-my-shell starts! ########");
    loop {
        print!("\n>>> ");
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
        let mut commands: Vec<String> = input.split("|").map(|x| x.trim().to_string()).collect();

        // Execute the parsed commands
        execute_command(commands).expect("Failed to execute command");
    }

    println!("Exit oh-my-shell. Bye!");
}
