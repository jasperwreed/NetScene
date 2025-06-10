import { useState } from "react";
import reactLogo from "./assets/react.svg";
import { invoke } from "@tauri-apps/api/core";
import "./App.css";

function App() {
  const [greetMsg, setGreetMsg] = useState("");
  const [name, setName] = useState("");
  const [devices, setDevices] = useState<{ ip: string; mac: string }[]>([]);

  async function greet() {
    // Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
    setGreetMsg(await invoke<string>("greet", { name }));
  }

  async function scan() {
    const result = await invoke<{ ip: string; mac: string }[]>("scan_network");
    setDevices(result);
  }

  return (
    <main>
      <h1>Welcome to Tauri + React</h1>

      <div>
        <a href="https://vite.dev" target="_blank">
          <img src="/vite.svg" className="logo vite" alt="Vite logo" />
        </a>
        <a href="https://tauri.app" target="_blank">
          <img src="/tauri.svg" className="logo tauri" alt="Tauri logo" />
        </a>
        <a href="https://react.dev" target="_blank">
          <img src={reactLogo} className="logo react" alt="React logo" />
        </a>
      </div>
      <p>Click on the Tauri, Vite, and React logos to learn more.</p>

      <form
        onSubmit={(e) => {
          e.preventDefault();
          greet();
        }}
      >
        <input
          id="greet-input"
          onChange={(e) => setName(e.currentTarget.value)}
          placeholder="Enter a name..."
        />
        <button type="submit">Greet</button>
      </form>
      <p>{greetMsg}</p>

      <button id="scan-button" onClick={scan}>Scan Network</button>
      <table>
        <thead>
          <tr>
            <th>IP Address</th>
            <th>MAC Address</th>
          </tr>
        </thead>
        <tbody>
          {devices.map((d) => (
            <tr key={d.ip}>
              <td>{d.ip}</td>
              <td>{d.mac}</td>
            </tr>
          ))}
        </tbody>
      </table>
    </main>
  );
}

export default App;
