use pty::fork::*;
use std::io::Read; // For reading from PTY master
use std::io::Write; // For writing to PTY master
use std::process::Command;

fn main() {
    // Create a PTY and fork the process
    let fork = Fork::from_ptmx().unwrap();

    if let Some(mut master) = fork.is_parent().ok() {
        // Parent process: Send keystrokes
        println!("Parent: sending commands to bash...");

        // Write commands (simulate keystrokes) to PTY master
        master.write_all(b"echo 'Hello from PTY'\n").unwrap();
        master.write_all(b"ls -la\n").unwrap();
        master.write_all(b"exit\n").unwrap();

        // Optionally read the output from the bash process
        let mut output = String::new();
        master.read_to_string(&mut output).unwrap();
        println!("Output from bash:\n{}", output);
    } else {
        // Child process: Run bash
        println!("Child: launching bash...");
        Command::new("bash")
            .status()
            .expect("failed to start bash");
    }
}

