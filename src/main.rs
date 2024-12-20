use io::{stdin, stdout};
use nix::sys::wait::{waitpid, WaitStatus};
use nix::unistd::{close, dup2, execvp, fork, pipe, ForkResult};
use std::ffi::CString;
use std::fs::File;
use std::io::{self, Write};
use std::os::fd::IntoRawFd;
use std::os::unix::io::RawFd;
use std::process::exit;

fn run_commands(mut commands: Vec<String>) -> nix::Result<()> {
    let mut pipe_previous: Option<(RawFd, RawFd)> = None;
    let mut fd_output: Option<RawFd> = None;
    let mut fd_input: Option<RawFd> = None;
    let mut children = Vec::new();
    let command_count = commands.len();

    if command_count == 1 {
        let output_position = commands[0].find(">");
        let input_position = commands[0].find("<");

        if let (Some(output_position), Some(input_position)) = (output_position, input_position) {
            if output_position > input_position {
                fd_input = Some(File::open(&commands[0][input_position + 1..output_position].trim())
                    .expect("file open failed")
                    .into_raw_fd());
                fd_output = Some(File::create(&commands[0][output_position + 1..].trim())
                    .expect("file creation failed")
                    .into_raw_fd());
                commands[0] = commands[0][..input_position].trim().to_string();
            } else {
                fd_input = Some(File::open(&commands[0][input_position + 1..].trim())
                    .expect("file open failed")
                    .into_raw_fd());
                fd_output = Some(File::create(&commands[0][output_position + 1..input_position].trim())
                    .expect("file creation failed")
                    .into_raw_fd());
                commands[0] = commands[0][..input_position].trim().to_string();
            }
        }
    }

    if let Some((command, file)) = commands[0].split_once(&String::from("<")) {
        fd_input = Some(File::open(file.trim()).expect("file open failed").into_raw_fd());
        commands[0] = command.trim().to_string();
    }
    if let Some((command, file)) = commands[command_count - 1].split_once(&String::from(">")) {
        fd_output = Some(File::create(file.trim()).expect("file creation failed").into_raw_fd());
        commands[command_count - 1] = command.trim().to_string();
    }

    for (i, command) in commands.into_iter().enumerate() {
        let pipe_next = if i < command_count - 1 {
            Some(pipe().map(|(x, y)| (x.into_raw_fd(), y.into_raw_fd()))?)
        } else { None };

        match unsafe { fork() } {
            Ok(ForkResult::Parent { child }) => {
                children.push(waitpid(child, None)?);

                if let Some((_, fd_write)) = pipe_next { close(fd_write)?; }
                if let Some((fd_read, _)) = pipe_previous { close(fd_read)?; }
                if fd_input.is_some() { fd_input = None; }

                pipe_previous = pipe_next;
            }
            Ok(ForkResult::Child) => {
                if let Some((read_fd, _)) = pipe_previous { dup2(read_fd, 0)?; }
                if let Some((_, write_fd)) = pipe_next { dup2(write_fd, 1)?; }

                if let Some(file_fd) = fd_input {
                    dup2(file_fd, 0)?;
                    close(file_fd)?;
                }
                if i == command_count - 1 && fd_output.is_some() {
                    dup2(fd_output.unwrap(), 1)?;
                    close(fd_output.unwrap())?;
                }

                let args = process_args(&command);
                let Err(error) = execvp(&args[0], &args);

                println!("execution failure: {}", error);
                exit(1);
            }
            Err(error) => {
                println!("failed to fork: {}", error);
            }
        }
    }

    if let Some(fd) = fd_output {
        close(fd)?;
    }
    if let Some((read_fd, write_fd)) = pipe_previous {
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
    let mut quote = false;
    let mut args = vec![];
    let mut current = String::new();

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

        input = input.trim().to_string();

        if input == "exit" { break; }
        if input.is_empty() { continue; }

        let commands: Vec<String> = input
            .split("|")
            .map(|x| x.trim().to_string())
            .collect();

        if let Err(_) = run_commands(commands) { continue; }
    }

    println!("Exit oh-my-shell. Bye!");
}