import { useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import "./App.css";

function App() {
  const [devices, setDevices] = useState<{ ip: string; mac: string }[]>([]);
  const [stats, setStats] = useState<{
    domains_being_blocked: number;
    dns_queries_today: number;
    ads_blocked_today: number;
    ads_percentage_today: number;
    status: string;
  } | null>(null);


  async function scan() {
    const result = await invoke<{ ip: string; mac: string }[]>("scan_network");
    setDevices(result);
  }

  async function fetchStats() {
    const result = await invoke<{
      domains_being_blocked: number;
      dns_queries_today: number;
      ads_blocked_today: number;
      ads_percentage_today: number;
      status: string;
    }>("get_pihole_stats", { host: "pi.hole" });
    setStats(result);
  }

  return (
    <main>
      <button id="scan-button" onClick={scan}>Scan Network</button>
      <button id="stats-button" onClick={fetchStats}>Get Pi-hole Stats</button>
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
      {stats && (
        <div id="pihole-stats">
          <h2>Pi-hole Stats</h2>
          <ul>
            <li>Domains blocked: {stats.domains_being_blocked}</li>
            <li>DNS queries today: {stats.dns_queries_today}</li>
            <li>Ads blocked today: {stats.ads_blocked_today}</li>
            <li>Ads percentage today: {stats.ads_percentage_today}%</li>
            <li>Status: {stats.status}</li>
          </ul>
        </div>
      )}
    </main>
  );
}

export default App;
