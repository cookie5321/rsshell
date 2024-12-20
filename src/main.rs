use io::{stdin, stdout};
use nix::sys::wait::{waitpid, WaitStatus};
use nix::unistd::{close, dup2, execvp, fork, pipe, ForkResult};
use std::ffi::CString;
use std::fs::File;
use std::io::{self, Write};
use std::os::fd::IntoRawFd;
use std::os::unix::io::RawFd;
use std::process::exit;

fn execute_command(mut commands: Vec<String>) -> nix::Result<()> {
    let mut prev_pipe: Option<(RawFd, RawFd)> = None;
    let mut output_fd: Option<RawFd> = None;
    let mut input_fd: Option<RawFd> = None;
    let mut children = Vec::new();

    if commands.len() == 1 {
        let output_position = commands[0].find(">");
        let input_position = commands[0].find("<");

        if let (Some(output_position), Some(input_position)) = (output_position, input_position) {
            if output_position > input_position {
                input_fd = Some(File::open(&commands[0][input_position + 1..output_position].trim())
                    .expect("file open failed")
                    .into_raw_fd());
                output_fd = Some(File::create(&commands[0][output_position + 1..].trim())
                    .expect("file creation failed")
                    .into_raw_fd());
                commands[0] = commands[0][..input_position].trim().to_string();
            } else {
                input_fd = Some(File::open(&commands[0][input_position + 1..].trim())
                    .expect("file open failed")
                    .into_raw_fd());
                output_fd = Some(File::create(&commands[0][output_position + 1..input_position].trim())
                    .expect("file creation failed")
                    .into_raw_fd());
                commands[0] = commands[0][..input_position].trim().to_string();
            }
        }
    }

    if let Some((command, file)) = commands[0].split_once(&String::from("<")) {
        input_fd = Some(File::open(file.trim()).expect("file open failed").into_raw_fd());
        commands[0] = command.trim().to_string();
    }
    if let Some((command, file)) = commands.last().unwrap().split_once(&String::from(">")) {
        output_fd = Some(File::create(file.trim()).expect("file creation failed").into_raw_fd());
        let len = commands.len();
        commands[len - 1] = command.trim().to_string();
    }

    for (i, command) in commands.iter().enumerate() {
        let next_pipe = if i < commands.len() - 1 {
            Some(pipe().map(|(x, y)| (x.into_raw_fd(), y.into_raw_fd()))?)
        } else { None };

        match unsafe { fork() } {
            Ok(ForkResult::Child) => {
                if i == 0 {
                    if let Some(file_fd) = input_fd {
                        dup2(file_fd, 0)?;
                        close(file_fd)?;
                    }
                }
                if i == commands.len() - 1 {
                    if let Some(output_fd) = output_fd {
                        dup2(output_fd, 1)?;
                        close(output_fd)?;
                    }
                }

                if let Some((read_fd, _)) = prev_pipe { dup2(read_fd, 0)?; }
                if let Some((_, write_fd)) = next_pipe { dup2(write_fd, 1)?; }

                let args = process_args(command);
                let Err(error) = execvp(&args[0], &args);

                println!("Execution failure: {}", error);
                exit(1);
            }
            Ok(ForkResult::Parent { child }) => {
                children.push(waitpid(child, None)?);

                if let Some((_, write_fd)) = next_pipe { close(write_fd)?; }
                if let Some((read_fd, _)) = prev_pipe { close(read_fd)?; }

                prev_pipe = next_pipe;
            }
            Err(err) => {
                println!("Failed to fork: {}", err);
            }
        }
    }

    if let Some(fd) = output_fd {
        close(fd)?;
    }
    if let Some((read_fd, write_fd)) = prev_pipe {
        close(read_fd)?;
        close(write_fd)?;
    }

    for child in children {
        println!("[oh-my-shell] Child process terminated: pid {}, {}",
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
    let mut args = vec![];
    let mut current = String::new();
    let mut quote = false;

    for c in input.chars() {
        if c == '"' { quote = !quote; }
        else if c == ' ' && !quote && !current.is_empty() {
            args.push(CString::new(current).unwrap());
            current = String::new();
        } else { current.push(c); }
    }

    if !current.is_empty() { args.push(CString::new(current).unwrap()); }

    args
}

fn main() {
    print!("######## oh-my-shell starts! ########");

    loop {
        print!("\n>>> ");
        stdout().flush().unwrap();

        let mut input = String::new();
        stdin().read_line(&mut input).unwrap();

        let input = input.trim();

        if input.is_empty() { continue; }
        if input == "exit" { break; }

        let commands: Vec<String> = input
            .split("|")
            .map(|x| x.trim().to_string())
            .collect();

        if let Err(_) = execute_command(commands) { continue; }
    }

    println!("Exit oh-my-shell. Bye!");
}
