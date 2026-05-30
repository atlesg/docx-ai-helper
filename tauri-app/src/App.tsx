import { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import "./App.css";

interface AppSettings {
  openai_api_key: string;
  gemini_api_key: string;
  active_provider: string;
  openai_model: string;
  gemini_model: string;
  port: number;
}

function App() {
  const [settings, setSettings] = useState<AppSettings>({
    openai_api_key: "",
    gemini_api_key: "",
    active_provider: "openai",
    openai_model: "gpt-4o-mini",
    gemini_model: "gemini-1.5-flash",
    port: 44320
  });

  const [maskedKey, setMaskedKey] = useState<string>("");
  const [showKey, setShowKey] = useState<boolean>(false);
  const [saveStatus, setSaveStatus] = useState<string>("");
  const [certStatus, setCertStatus] = useState<string>("");
  const [isTrusting, setIsTrusting] = useState<boolean>(false);
  const [isSaving, setIsSaving] = useState<boolean>(false);

  // Fetch settings on load
  useEffect(() => {
    loadSettings();
  }, []);

  const loadSettings = async () => {
    try {
      const currentSettings = await invoke<AppSettings>("get_app_settings");
      setSettings(currentSettings);
      
      const key = currentSettings.active_provider === "openai" 
        ? currentSettings.openai_api_key 
        : currentSettings.gemini_api_key;
      setMaskedKey(key);
    } catch (e) {
      console.error("Failed to load settings:", e);
    }
  };

  const handleProviderChange = (provider: string) => {
    setSettings(prev => {
      const updated = { ...prev, active_provider: provider };
      const key = provider === "openai" ? updated.openai_api_key : updated.gemini_api_key;
      setMaskedKey(key);
      return updated;
    });
  };

  const handleKeyChange = (value: string) => {
    setMaskedKey(value);
    setSettings(prev => {
      if (prev.active_provider === "openai") {
        return { ...prev, openai_api_key: value };
      } else {
        return { ...prev, gemini_api_key: value };
      }
    });
  };

  const handleModelChange = (value: string) => {
    setSettings(prev => {
      if (prev.active_provider === "openai") {
        return { ...prev, openai_model: value };
      } else {
        return { ...prev, gemini_model: value };
      }
    });
  };

  const handleSave = async () => {
    setIsSaving(true);
    setSaveStatus("Saving...");
    try {
      await invoke("save_app_settings", { settings });
      setSaveStatus("Settings saved successfully!");
      setTimeout(() => setSaveStatus(""), 3000);
      await loadSettings();
    } catch (e) {
      setSaveStatus(`Error: ${e}`);
    } finally {
      setIsSaving(false);
    }
  };

  const handleTrustCert = async () => {
    setIsTrusting(true);
    setCertStatus("Trusting certificate...");
    try {
      const result = await invoke<string>("run_trust_cert");
      setCertStatus(result);
    } catch (e) {
      setCertStatus(`Trust failed: ${e}`);
    } finally {
      setIsTrusting(false);
    }
  };

  const activeModels = settings.active_provider === "openai" 
    ? [
        { value: "gpt-4o-mini", label: "GPT-4o Mini (Fast, Recommended)" },
        { value: "gpt-4o", label: "GPT-4o (High Quality)" },
        { value: "o1-mini", label: "o1 Mini (Reasoning)" }
      ]
    : [
        { value: "gemini-1.5-flash", label: "Gemini 1.5 Flash (Fast)" },
        { value: "gemini-1.5-pro", label: "Gemini 1.5 Pro (Rich Context)" },
        { value: "gemini-2.0-flash", label: "Gemini 2.0 Flash (Experimental)" },
        { value: "gemini-2.5-flash", label: "Gemini 2.5 Flash" },
        { value: "gemini-2.5-pro", label: "Gemini 2.5 Pro" }
      ];

  const currentModelValue = settings.active_provider === "openai"
    ? settings.openai_model
    : settings.gemini_model;

  return (
    <div className="settings-container">
      {/* Header */}
      <header className="dashboard-header">
        <div className="logo-section">
          <div className="logo-glow"></div>
          <svg className="logo-icon" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5">
            <path d="M12 2L2 7l10 5 10-5-10-5zM2 17l10 5 10-5M2 12l10 5 10-5"/>
          </svg>
          <h1>DOCX <span>AI Helper Settings</span></h1>
        </div>
        <div className="server-status-pill">
          <span className="status-dot active"></span>
          <span className="status-label">Local Server: <span>HTTPS 44320</span></span>
        </div>
      </header>

      {/* Main Grid */}
      <main className="dashboard-grid">
        {/* Left Side: Credentials */}
        <section className="glass-card">
          <h2>
            <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
              <rect x="3" y="11" width="18" height="11" rx="2" ry="2"/>
              <path d="M7 11V7a5 5 0 0110 0v4"/>
            </svg>
            API Credentials
          </h2>

          <div className="form-group">
            <label>API Provider</label>
            <div className="provider-toggle-group">
              <button 
                className={`provider-toggle-btn ${settings.active_provider === "openai" ? "active" : ""}`}
                onClick={() => handleProviderChange("openai")}
              >
                OpenAI
              </button>
              <button 
                className={`provider-toggle-btn ${settings.active_provider === "gemini" ? "active" : ""}`}
                onClick={() => handleProviderChange("gemini")}
              >
                Google Gemini
              </button>
            </div>
          </div>

          <div className="form-group">
            <label>API Key</label>
            <div className="input-container">
              <input 
                type={showKey ? "text" : "password"} 
                value={maskedKey}
                onChange={(e) => handleKeyChange(e.target.value)}
                placeholder={settings.active_provider === "openai" ? "sk-..." : "AIzaSy..."}
              />
              <button 
                className="mask-toggle" 
                onClick={() => setShowKey(!showKey)}
                title={showKey ? "Hide key" : "Show key"}
              >
                <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                  <path d="M1 12s4-8 11-8 11 8 11 8-4 8-11 8-11-8-11-8z"/>
                  <circle cx="12" cy="12" r="3"/>
                </svg>
              </button>
            </div>
          </div>

          <div className="form-group">
            <label>AI Model</label>
            <select 
              value={currentModelValue} 
              onChange={(e) => handleModelChange(e.target.value)}
            >
              {activeModels.map(model => (
                <option key={model.value} value={model.value}>
                  {model.label}
                </option>
              ))}
            </select>
          </div>

          <button 
            className="action-btn" 
            onClick={handleSave}
            disabled={isSaving}
          >
            {isSaving ? "Saving..." : "Save Credentials"}
          </button>
          
          {saveStatus && <p className="status-card-info" style={{ textAlign: "center", color: saveStatus.includes("Error") ? "#EF4444" : "#10B981" }}>{saveStatus}</p>}
        </section>

        {/* Right Side: Setup Instructions */}
        <section className="glass-card">
          <h2>
            <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
              <path d="M14.7 6.3a1 1 0 000 1.4l1.6 1.6a1 1 0 001.4 0l3.77-3.77a6 6 0 01-7.94 7.94l-6.91 6.91a2.12 2.12 0 01-3-3l6.91-6.91a6 6 0 017.94-7.94l-3.76 3.76z"/>
            </svg>
            Office Add-In Onboarding
          </h2>

          <div className="help-list">
            <div className="help-item">
              <div className="help-number">1</div>
              <div className="help-text">
                <strong>Trust local connection:</strong> Click the button below to trust our local self-signed certificate so Word can load the task pane.
              </div>
            </div>
            
            <button 
              className="action-btn-secondary" 
              onClick={handleTrustCert}
              disabled={isTrusting}
            >
              {isTrusting ? "Trusting..." : "Trust Self-Signed SSL Cert"}
            </button>
            {certStatus && <p className="status-card-info" style={{ color: certStatus.includes("failed") ? "#EF4444" : "#10B981", fontSize: "11px" }}>{certStatus}</p>}

            <div className="help-item">
              <div className="help-number">2</div>
              <div className="help-text">
                <strong>Register manifest in Word:</strong> Sideload the <strong>manifest.xml</strong> located in the installation folder into Word. (Go to Word → Insert → My Add-ins → Upload Manifest).
              </div>
            </div>

            <div className="help-item">
              <div className="help-number">3</div>
              <div className="help-text">
                <strong>Power up!</strong> Select any text or table in Word, click the <strong>AI Helper Panel</strong> in the Home Ribbon, and prompt the AI.
              </div>
            </div>
          </div>
        </section>
      </main>
    </div>
  );
}

export default App;
