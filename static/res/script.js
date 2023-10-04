const URL = 'ws://localhost:8000/ws';

let connected = false;
let room = null;

let connectionIndicator;
let usernameText;
let content;
let loginBox;
let joinRoomBox;
let chatScreen;
let inner;
let roomCodeLabel;

let usernameInput;
let roomCodeInput;
let chatBoxInput;

let username = null;

let colors = new Map();

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

function resetColors() {
  colors = new Map();
}

const HSLToRGB = (h, s, l) => {
  s /= 100;
  l /= 100;
  const k = n => (n + h / 30) % 12;
  const a = s * Math.min(l, 1 - l);
  const f = n =>
    l - a * Math.max(-1, Math.min(k(n) - 3, Math.min(9 - k(n), 1)));
  return [255 * f(0), 255 * f(8), 255 * f(4)];
};

function componentToHex(c) {
  c = Math.floor(c);
  var hex = c.toString(16);
  return hex.length == 1 ? "0" + hex : hex;
}

function rgbToHex(r, g, b) {
  return "#" + componentToHex(r) + componentToHex(g) + componentToHex(b);
}

function createNewColor() {
  let h = Math.floor(Math.random() * 255);
  let s = Math.floor(Math.random() * 25 + 55);
  let l = Math.floor(Math.random() * 10 + 70);
  let [r, g, b] = HSLToRGB(h, s, l);
  let color = rgbToHex(r, g, b);
  return color;
}

function addChatMessage(user, message) {
  console.log(`message received from ${username}: ${message}`);
  let container = document.createElement('div');
  container.classList.add('chat');
  if (user == username) {
    container.classList.add("gray");
    container.classList.add("outgoing");
  } else {
    container.classList.add("incoming");
    if (colors.has(user)) {
      container.style.backgroundColor = colors.get(user);
    } else {
      let color = createNewColor();
      container.style.backgroundColor = color;
      colors.set(user, color);
    }
  }
  let usernameElement = document.createElement('h4');
  usernameElement.className = "username";
  usernameElement.innerHTML = user;
  let messageElement = document.createElement('pre');
  messageElement.className = "message";
  messageElement.innerHTML = message;
  container.appendChild(usernameElement);
  container.appendChild(messageElement);
  inner.appendChild(container);
}

function addNotification(user, leave) {
  let container = document.createElement('span');
  container.className = 'join-leave-notification';
  let joinStatus = leave ? 'left' : 'joined';
  container.innerHTML = `${user} ${joinStatus} the chat`;
  inner.appendChild(container);
}

function userLeft(user) {
  addNotification(user, true);
  if (user == username) {
    gotoPage('joinRoom');
    room = null;
  }
}

function userJoined(user, code) {
  addNotification(user, false);
  if (user == username) {
    gotoPage('chat');
    room = code;
    roomCodeLabel.innerHTML = "(" + code +")";
  }
}

function showError(error) {
  alert(error);
}

function keypress(e, callback) {
  if (e.shiftKey) {
    return;
  } if (e.key == "Enter") {
    callback();
  }
}

function gotoPage(page) {
  switch (page) {
    case "chat":
      content.classList.add("hidden");
      loginBox.classList.add("hidden");
      joinRoomBox.classList.add("hidden");
      chatScreen.classList.remove("hidden");
      inner.innerHTML = "";
      break;
    case "login":
      content.classList.remove("hidden");
      loginBox.classList.remove("hidden");
      joinRoomBox.classList.add("hidden");
      chatScreen.classList.add("hidden");
      break;
    case "joinRoom":
      content.classList.remove("hidden");
      loginBox.classList.add("hidden");
      joinRoomBox.classList.remove("hidden");
      chatScreen.classList.add("hidden");
      break;
    default:
      alert("no such page: " + page);
      break;
  }
}

function updateConnectionStatus(active, username) {
  if (active) {
    connectionIndicator.classList.replace('inactive', 'active');
    connected = true;
    if (username) {
      usernameText.innerHTML = username;
    } else {
      usernameText.innerHTML = "connected";
    }
  } else {
    connected = false;
    connectionIndicator.classList.replace('active', 'inactive');
    usernameText.innerHTML = "disconnected";
  }
}

function handleMessage(message) {
  console.debug(message.data);
  const parsed = JSON.parse(message.data);
  if (parsed == "RemoveFromRoom") {
    room = null;
    gotoPage('joinRoom');
  } else if (parsed == "LoginSuccess") {
    updateConnectionStatus(true, username);
    gotoPage('joinRoom');
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
    socket.onopen = (_) => updateConnectionStatus(true);
    socket.onerror = (err) => alert('error in websocket: ' + err);
    socket.onclose = (reason) => {
      updateConnectionStatus(false);
      alert('connection terminated: ' + reason);
    }
    socket.onmessage = (message) => handleMessage(message);
    sendJson = (object) => {
      socket.send(JSON.stringify(object));
    }
    send = (string) => {
      socket.send(string);
    }
  } catch (err) {
    connected = false;
    alert(err);
  }
}

function login() {
  if (!connected) {
    alert('not connected!');
    return;
  }
  if (usernameInput.reportValidity()) {
    username = usernameInput.value;
    let loginMessage = {
      Login: username
    };
    sendJson(loginMessage);
  }
}

function joinRoom() {
  if (!connected) {
    alert('not connected!');
    return;
  }
  if (roomCodeInput.reportValidity()) {
    let roomCode = roomCodeInput.value;
    let joinRoomMessage = {
      JoinRoom: roomCode
    };
    sendJson(joinRoomMessage);
  }
}

function createRoom() {
  if (!connected) {
    alert('not connected!');
    return;
  }
  if (username) {
    let createRoomMessage = "CreateRoom";
    sendJson(createRoomMessage);
  } else {
    alert('not signed in!');
    return;
  }
}

function sendChatMessage() {
  if (!connected || !username) {
    alert('not connected!');
    return;
  }
  if (!room) {
    alert('not in room!');
    return;
  } else {
    if (chatBoxInput.reportValidity()) {
      let message = chatBoxInput.value;
      let chatMessage = {
        Message: message
      };
      sendJson(chatMessage);
      chatBoxInput.value = "";
    }
  }
}

function leaveRoom() {
  if (!room || !connected || !username) {
    alert('cannot do that!');
    return;
  }
  let leaveRoomMessage = "LeaveRoom";
  sendJson(leaveRoomMessage);
  room = null;
}

function init() {
  connectionIndicator = document.querySelector("#connection-indicator");
  usernameText = document.querySelector("#connection-status");
  content = document.querySelector(".content");
  loginBox = document.querySelector("#login-page");
  joinRoomBox = document.querySelector("#join-room-page");
  chatScreen = document.querySelector("#chat");
  usernameInput = document.querySelector("#username-input");
  roomCodeInput = document.querySelector("#room-code-input");
  chatBoxInput = document.querySelector("#chat-input-box");
  inner = document.querySelector('#chat-list');
  roomCodeLabel = document.querySelector('#code');
  gotoPage('login');
  connect();
  resetColors();
}

init();
