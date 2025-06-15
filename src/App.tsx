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
  const [piholeHost, setPiholeHost] = useState<string>("pi.hole");
  const [piholePassword, setPiholePassword] = useState<string>("");
  const [statsLoading, setStatsLoading] = useState<boolean>(false);
  const [statsError, setStatsError] = useState<string>("");

  async function scan() {
    const result = await invoke<{ ip: string; mac: string }[]>("scan_network");
    setDevices(result);
  }

  async function fetchStats() {
    if (!piholeHost.trim()) {
      setStatsError("Please enter a Pi-hole host");
      return;
    }

    setStatsLoading(true);
    setStatsError("");
    setStats(null);

    try {
      const result = await invoke<{
        domains_being_blocked: number;
        dns_queries_today: number;
        ads_blocked_today: number;
        ads_percentage_today: number;
        status: string;
      }>("get_pihole_stats", { 
        host: piholeHost.trim(),
        password: piholePassword.trim() || null
      });
      setStats(result);
    } catch (error) {
      console.error("Failed to fetch Pi-hole stats:", error);
      setStatsError(`Failed to fetch Pi-hole stats: ${error}`);
    } finally {
      setStatsLoading(false);
    }
  }

  return (
    <main>
      <button id="scan-button" onClick={scan}>Scan Network</button>
      
      <div id="pihole-section">
        <h2>Pi-hole Stats</h2>
        <div id="pihole-inputs">
          <label>
            Pi-hole Host:
            <input
              type="text"
              value={piholeHost}
              onChange={(e) => setPiholeHost(e.target.value)}
              placeholder="pi.hole or 192.168.1.100"
            />
          </label>
          <label>
            Password (optional):
            <input
              type="password"
              value={piholePassword}
              onChange={(e) => setPiholePassword(e.target.value)}
              placeholder="Leave empty if no password required"
            />
          </label>
        </div>
        <button 
          id="stats-button" 
          onClick={fetchStats} 
          disabled={statsLoading || !piholeHost.trim()}
        >
          {statsLoading ? "Loading..." : "Get Pi-hole Stats"}
        </button>
        
        {statsError && (
          <div id="stats-error" style={{ color: "red", marginTop: "10px" }}>
            {statsError}
          </div>
        )}
        
        {stats && (
          <div id="pihole-stats">
            <h3>Results</h3>
            <ul>
              <li>Domains blocked: {stats.domains_being_blocked.toLocaleString()}</li>
              <li>DNS queries today: {stats.dns_queries_today.toLocaleString()}</li>
              <li>Ads blocked today: {stats.ads_blocked_today.toLocaleString()}</li>
              <li>Ads percentage today: {stats.ads_percentage_today.toFixed(2)}%</li>
              <li>Status: {stats.status}</li>
            </ul>
          </div>
        )}
      </div>
      
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
