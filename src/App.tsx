import { useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import "./App.css";

function App() {
  const [devices, setDevices] = useState<{ ip: string; mac: string }[]>([]);


  async function scan() {
    const result = await invoke<{ ip: string; mac: string }[]>("scan_network");
    setDevices(result);
  }

  return (
    <main>
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
