import os from 'os';
import pty from 'node-pty';

let sharedPtyProcess = null;
let sharedTerminalMode = false;

const shell = os.platform() === 'win32' ? 'powershell.exe' : 'bash';

const spawnShell = () => {
    return pty.spawn(shell, [], {
        name: 'xterm-256color',
        env: process.env,
    });
};

export const setSharedTerminalMode = (useSharedTerminal) => {
    sharedTerminalMode = useSharedTerminal;
    if (sharedTerminalMode && !sharedPtyProcess) {
        sharedPtyProcess = spawnShell();
    }
};

export const handleTerminalConnection = (ws) => {
    let ptyProcess = sharedTerminalMode ? sharedPtyProcess : spawnShell();

    ws.on('message', command => {
        const processedCommand = commandProcessor(command, ptyProcess);
        ptyProcess.write(processedCommand);
    });

    ptyProcess.on('data', (rawOutput) => {
        const processedOutput = outputProcessor(rawOutput);
        ws.send(processedOutput);
    });

    ws.on('close', () => {
        if (!sharedTerminalMode) {
            ptyProcess.kill();
        }
    });
};

// Utility function to process commands
const commandProcessor = (command, proc) => {
  console.log('commandProcessor', command.toString());
  command = JSON.parse(command.toString());
  if (command.event === 'command') {
    return command.content;
  } else if (command.event === 'resize') {
    if (!proc) {
      console.warn('Resize event received, but sharedPtyProcess is not initialized.');
    } else {
      proc.resize(command.content.cols, command.content.rows);
      console.log('Resized successfully.');
    }
  }
  return '';
};

// Utility function to process output
const outputProcessor = (output) => {
    return output;
};
