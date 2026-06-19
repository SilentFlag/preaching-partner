<main class="container">
  <h1>Welcome to Ministry Manager</h1>
  <p>Enter the details provided below to log in.</p>
  <p id="login-failed" style="color: red;">{login_failed}</p>
  <div class="collumn form" style="margin-top: 2em;">
    <input bind:value={username} id="name-input" type="text" placeholder="Name" />
    <input bind:value={serverAddress} id="server-input" type="text" placeholder="Server Address" />
    <input bind:value={password} id="password-input" type="password" placeholder="Code" />
    <button id="greet-button" onclick={handleLogin}>Log In</button>
  </div>
</main>

<script>
  import { invoke } from '@tauri-apps/api/core';
  import { listen } from '@tauri-apps/api/event';

  let username = $state('username');
  let serverAddress = $state('serverAddress');
  let password = $state('password');
  let login_failed = $state('');

  listen('login', event => {
    let success = event.payload;
    console.log('Received login event with success:', success);
    if (success  === true) {
      console.log('Login successful, navigating to dashboard');
      window.location.replace('./dashboard');
    } else {
      console.log('Login failed, showing error message');
      login_failed = 'Invalid username or password';
    }
  });

  // TODO: listen for an event to hide a loading screen rather than having the form by default
  // listen('form', event => {
  //   console.log('Received login event:', event);
  //   window.location.replace('./dashboard');
  // });

  async function handleLogin() {

    try {
      console.log('Login call successful');
      await invoke('login', { username, password });
    } catch (error) {
      console.error('Login failed', error);
    }
  }

  // TODO: Need to invoke this when the app is done loading, not when the page is loaded
  invoke("app_loaded");
</script>

<style>

:root {
  font-family: Inter, Avenir, Helvetica, Arial, sans-serif;
  font-size: 16px;
  line-height: 24px;
  font-weight: 400;

  color: #0f0f0f;
  background-color: #f6f6f6;

  font-synthesis: none;
  text-rendering: optimizeLegibility;
  -webkit-font-smoothing: antialiased;
  -moz-osx-font-smoothing: grayscale;
  -webkit-text-size-adjust: 100%;
}

.container {
  margin: 0;
  padding-top: 10vh;
  display: flex;
  flex-direction: column;
  justify-content: center;
  text-align: center;
}

.collumn {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
}

.form {
  max-width: 400px;
  width: 55%;
  margin: 0 auto;
}

h1 {
  text-align: center;
}

input,
button {
  border-radius: 8px;
  border: 1px solid transparent;
  padding: 0.6em 1.2em;
  font-size: 1em;
  font-weight: 500;
  font-family: inherit;
  color: #0f0f0f;
  background-color: #ffffff;
  transition: border-color 0.25s;
  box-shadow: 0 2px 2px rgba(0, 0, 0, 0.2);
  margin: 5px;
  width: 100%;
  box-sizing: border-box;
}

button {
  cursor: pointer;
}

button:hover {
  border-color: #396cd8;
}
button:active {
  border-color: #396cd8;
  background-color: #e8e8e8;
}

input,
button {
  outline: none;
}

.form input {
  margin-right: 5px;
}

@media (prefers-color-scheme: dark) {
  :root {
    color: #f6f6f6;
    background-color: #2f2f2f;
  }

  input,
  button {
    color: #ffffff;
    background-color: #0f0f0f98;
  }
  button:active {
    background-color: #0f0f0f69;
  }
}

</style>
