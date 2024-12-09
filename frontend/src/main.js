import './style.css';
import { Terminal } from '@xterm/xterm';
import { FitAddon } from '@xterm/addon-fit';
import { WebglAddon } from '@xterm/addon-webgl';
import '@xterm/xterm/css/xterm.css';
import './style.css';

const theme = {
   foreground: '#d2d2d2',
   background: '#2b2b2b',
   cursor: '#adadad',
   black: '#000000',
   red: '#d81e00',
   green: '#5ea702',
   yellow: '#cfae00',
   blue: '#427ab3',
   magenta: '#89658e',
   cyan: '#00a7aa',
   white: '#dbded8',
   brightBlack: '#686a66',
   brightRed: '#f54235',
   brightGreen: '#99e343',
   brightYellow: '#fdeb61',
   brightBlue: '#84b0d8',
   brightMagenta: '#bc94b7',
   brightCyan: '#37e6e8',
   brightWhite: '#f1f1f0',
};

const terminal = new Terminal({
  fontFamily: 'DaddyTimeMono',
  theme: theme,
});
const fitAddon = new FitAddon();
terminal.loadAddon(fitAddon);
terminal.loadAddon(new WebglAddon());

const terminalElement = document.getElementById('terminal');

// Wait for the terminal element to be properly styled
terminal.open(terminalElement);
fitAddon.fit();

const socket = new WebSocket('ws://localhost:8080/connect');

socket.onmessage = (event) => {
  terminal.write(event.data);
}

socket.onopen = () => {
  init();
  runCommand('');
}

terminal.open(document.getElementById('terminal'));

function init() {
  if (terminal._initialized) {
      return;
  }

  terminal._initialized = true;

  terminal.onKey(keyObj => {
      runCommand(keyObj.key);
  });

  terminal.attachCustomKeyEventHandler((e) => {
      if ((e.ctrlKey || e.metaKey) && e.key === 'v') {
          navigator.clipboard.readText().then(text => {
              runCommand(text);
          });
          return false;
      }
      return true;
  });
}

function runCommand(command) {
  socket.send(JSON.stringify({
    "event": "command",
    "content": command
  }));
}

window.addEventListener('resize', () => {
  fitAddon.fit();
  socket.send(JSON.stringify({
    "event": "resize",
    "content": {
      "cols": terminal.cols,
      "rows": terminal.rows
    }
  }));
});
