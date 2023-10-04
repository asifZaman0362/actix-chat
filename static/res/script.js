const URL = 'ws://localhost:8000/ws';

/*
  *
pub enum OutgoingMessage {
    Chat { username: String, message: String },
    RemoveFromRoom,
    Error(SuperDuperError),
    UserJoined { username: String, room: String },
    UserLeft(String),
    LoginSuccess,
}
  */

function addChatMessage(message, username) {
}

function userLeft(username) {
}

function userJoined(username, room) {
}

function showError(error) {
}

function leaveRoom() {
}

function gotoPage(page) {
}

function updateConnectionStatus(active) {
}

function handleMessage(message) {
  const parsed = JSON.parse(message);
  console.debug(message);
  if (parsed == "RemoveFromRoom") {
    gotoPage('joinRoom');
  } else if (parsed == "LoginSuccess") {
    updateConnectionStatus(true);
  } else if (parsed.Chat != undefined) {
    addChatMessage(parsed.Chat.username, parsed.Chat.message);
  } else if (parsed.Error != undefined) {
    showError(parsed.Error);
  } else if (parsed.UserJoined != undefined) {
    userJoined(parsed.UserJoined.username, parsed.UserJoined.room);
  } else if (parsed.UserLeft != undefined) {
    userLeft(parsed.UserLeft);
  }
}

let sendJson = null;
let send = null;

function connect() {
  try {
    let socket = new WebSocket(URL);
    if (socket == null) {
      alert('failed to connect!');
    }
    socket.onerror = (err) => alert('error in websocket: ' + err);
    socket.onclose = (reason) => alert('connection terminated: ' + reason);
    socket.onmessage = (message) => handleMessage(message);
    sendJson = (object) => {
      socket.send(JSON.stringify(object));
    }
    send = (string) => {
      socket.send(string);
    }
  } catch (err) {
    alert(err);
  }
}

function login() {
}
