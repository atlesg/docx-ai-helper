// Word Add-In Core Logic
const LOCAL_SERVER = "https://localhost:44320";

let isConnected = false;
let currentSelection = "";
let aiResponseContent = "";
let aiResponseFormat = "text";

// Model lists
const PROVIDER_MODELS = {
    openai: [
        { value: "gpt-4o-mini", label: "GPT-4o Mini (Fast, Recommended)" },
        { value: "gpt-4o", label: "GPT-4o (High Quality)" },
        { value: "o1-mini", label: "o1 Mini (Reasoning)" }
    ],
    gemini: [
        { value: "gemini-1.5-flash", label: "Gemini 1.5 Flash (Fast)" },
        { value: "gemini-1.5-pro", label: "Gemini 1.5 Pro (Rich Context)" },
        { value: "gemini-2.0-flash", label: "Gemini 2.0 Flash (Experimental)" },
        { value: "gemini-2.5-flash", label: "Gemini 2.5 Flash" },
        { value: "gemini-2.5-pro", label: "Gemini 2.5 Pro" }
    ]
};

// Initialize Office Add-in
Office.onReady((info) => {
    if (info.host === Office.HostType.Word) {
        console.log("Office.js is ready inside Microsoft Word.");
        // Retrieve initial selection
        refreshSelection();
    } else {
        console.log("Running outside Microsoft Word (Web Browser mode).");
    }
    
    // Initialize UI and connection
    initApp();
});

async function initApp() {
    setupTabSwitching();
    setupProviderToggle();
    
    // Check connection with local Tauri server
    await checkConnection();
    
    // If connected, fetch current credentials
    if (isConnected) {
        await loadCredentials();
    }
    
    // Setup button listeners
    document.getElementById("btnRefreshSelection").addEventListener("click", refreshSelection);
    document.getElementById("btnSubmit").addEventListener("click", handleSubmitPrompt);
    document.getElementById("btnDiscard").addEventListener("click", handleDiscard);
    document.getElementById("btnReplace").addEventListener("click", handleReplaceSelection);
    document.getElementById("btnSaveSettings").addEventListener("click", handleSaveSettings);
    document.getElementById("btnTrustCert").addEventListener("click", handleTrustCert);
    document.getElementById("btnToggleMask").addEventListener("click", handleToggleMask);
}

// Health check with local server
async function checkConnection() {
    const statusDot = document.getElementById("statusDot");
    const statusText = document.getElementById("statusText");
    
    try {
        const response = await fetch(`${LOCAL_SERVER}/api/health`, { method: 'GET' });
        if (response.ok) {
            isConnected = true;
            statusDot.className = "status-dot connected";
            statusText.textContent = "Connected Locally";
        } else {
            throw new Error();
        }
    } catch (e) {
        isConnected = false;
        statusDot.className = "status-dot disconnected";
        statusText.textContent = "Offline (Start Helper)";
    }
}

// Setup simple Tab navigation
function setupTabSwitching() {
    const tabs = document.querySelectorAll(".tab-btn");
    const panels = document.querySelectorAll(".tab-panel");
    
    tabs.forEach(tab => {
        tab.addEventListener("click", () => {
            tabs.forEach(t => t.classList.remove("active"));
            panels.forEach(p => p.classList.remove("active"));
            
            tab.classList.add("active");
            const activePanel = document.getElementById(tab.getAttribute("data-tab"));
            activePanel.classList.add("active");
            
            // Check connection when toggling tabs
            checkConnection();
        });
    });
}

// Setup Settings Provider Buttons
function setupProviderToggle() {
    const provOpenAI = document.getElementById("provOpenAI");
    const provGemini = document.getElementById("provGemini");
    const modelSelect = document.getElementById("modelSelect");
    
    function setProvider(provider) {
        if (provider === "openai") {
            provOpenAI.classList.add("active");
            provGemini.classList.remove("active");
        } else {
            provGemini.classList.add("active");
            provOpenAI.classList.remove("active");
        }
        
        // Populate model choices
        modelSelect.innerHTML = "";
        PROVIDER_MODELS[provider].forEach(model => {
            const opt = document.createElement("option");
            opt.value = model.value;
            opt.textContent = model.label;
            modelSelect.appendChild(opt);
        });
    }
    
    provOpenAI.addEventListener("click", () => setProvider("openai"));
    provGemini.addEventListener("click", () => setProvider("gemini"));
    
    // Default load
    setProvider("openai");
}

// Mask/Unmask API Key toggle
function handleToggleMask() {
    const input = document.getElementById("apiKeyInput");
    if (input.type === "password") {
        input.type = "text";
    } else {
        input.type = "password";
    }
}

// Load current settings from local Server
async function loadCredentials() {
    try {
        const response = await fetch(`${LOCAL_SERVER}/api/settings`);
        if (response.ok) {
            const settings = await response.json();
            
            // Set active provider toggle
            const provider = settings.active_provider || "openai";
            if (provider === "openai") {
                document.getElementById("provOpenAI").click();
                if (settings.openai_model) document.getElementById("modelSelect").value = settings.openai_model;
            } else {
                document.getElementById("provGemini").click();
                if (settings.gemini_model) document.getElementById("modelSelect").value = settings.gemini_model;
            }
            
            // Set masked key
            const key = provider === "openai" ? settings.openai_api_key : settings.gemini_api_key;
            document.getElementById("apiKeyInput").value = key || "";
        }
    } catch (e) {
        console.error("Failed to load settings from server:", e);
    }
}

// Save credentials to local server
async function handleSaveSettings() {
    if (!isConnected) {
        alert("Cannot connect to local helper app. Please make sure the DOCX AI Helper is running.");
        return;
    }
    
    const activeProviderBtn = document.querySelector(".provider-btn.active");
    const activeProvider = activeProviderBtn.getAttribute("data-provider");
    const apiKey = document.getElementById("apiKeyInput").value;
    const modelSelect = document.getElementById("modelSelect").value;
    
    const settingsPayload = {
        openai_api_key: activeProvider === "openai" ? apiKey : "********",
        gemini_api_key: activeProvider === "gemini" ? apiKey : "********",
        active_provider: activeProvider,
        openai_model: activeProvider === "openai" ? modelSelect : "",
        gemini_model: activeProvider === "gemini" ? modelSelect : "",
        port: 44320
    };
    
    try {
        const btn = document.getElementById("btnSaveSettings");
        btn.textContent = "Saving...";
        btn.disabled = true;
        
        const response = await fetch(`${LOCAL_SERVER}/api/settings`, {
            method: "POST",
            headers: { "Content-Type": "application/json" },
            body: JSON.stringify(settingsPayload)
        });
        
        const res = await response.json();
        if (response.ok && res.status === "success") {
            alert("Credentials saved securely locally!");
            await loadCredentials();
        } else {
            alert(`Error: ${res.message}`);
        }
    } catch (e) {
        alert("Failed to save settings: " + e.message);
    } finally {
        const btn = document.getElementById("btnSaveSettings");
        btn.textContent = "Save Credentials";
        btn.disabled = false;
    }
}

// Trust local SSL cert call
async function handleTrustCert() {
    if (!isConnected) {
        alert("Cannot connect to local helper app.");
        return;
    }
    
    try {
        const response = await fetch(`${LOCAL_SERVER}/api/trust-cert`, { method: "POST" });
        const res = await response.json();
        if (response.ok && res.status === "success") {
            alert("Success! " + res.message);
        } else {
            alert("Certificate trust failed: " + res.message);
        }
    } catch (e) {
        alert("Error calling trust certificate: " + e.message);
    }
}

// Refresh document selection preview
async function refreshSelection() {
    const previewEl = document.getElementById("selectionText");
    
    // Check if Office/Word context is available
    if (typeof Word === "undefined" || !Office.context || !Office.context.document) {
        previewEl.textContent = "[Running outside Word] Sample text selection.";
        currentSelection = "Sample text selection.";
        return;
    }
    
    previewEl.textContent = "Reading Word selection...";
    
    try {
        await Word.run(async (context) => {
            const selection = context.document.getSelection();
            selection.load("text");
            await context.sync();
            
            const txt = selection.text.trim();
            if (txt) {
                currentSelection = txt;
                previewEl.textContent = txt;
                previewEl.classList.remove("placeholder-text");
            } else {
                currentSelection = "";
                previewEl.textContent = "No text selected in Word. Please select some text first.";
                previewEl.classList.add("placeholder-text");
            }
        });
    } catch (error) {
        previewEl.textContent = "Error reading selection: " + error.message;
        previewEl.classList.add("placeholder-text");
    }
}

// Submit prompt to local server
async function handleSubmitPrompt() {
    const prompt = document.getElementById("promptInput").value.trim();
    if (!prompt) {
        alert("Please enter a prompt for the AI.");
        return;
    }
    
    // Auto-refresh selection to capture latest
    await refreshSelection();
    
    if (!currentSelection) {
        alert("Please select some text or content in Word first.");
        return;
    }
    
    if (!isConnected) {
        alert("Offline: Please run the local DOCX AI Helper background application.");
        return;
    }
    
    const panel = document.getElementById("promptTab");
    const responseCard = document.getElementById("responseCard");
    const previewBox = document.getElementById("responsePreview");
    const badge = document.getElementById("formatBadge");
    
    try {
        panel.classList.add("loading-pulse");
        responseCard.style.display = "none";
        
        const response = await fetch(`${LOCAL_SERVER}/api/prompt`, {
            method: "POST",
            headers: { "Content-Type": "application/json" },
            body: JSON.stringify({
                selected_content: currentSelection,
                prompt: prompt
            })
        });
        
        const data = await response.json();
        if (response.ok) {
            aiResponseContent = data.modified_content;
            aiResponseFormat = data.format;
            
            badge.textContent = aiResponseFormat.toUpperCase();
            
            // Format layout inside taskpane preview
            if (aiResponseFormat === "html") {
                previewBox.innerHTML = aiResponseContent;
            } else {
                previewBox.textContent = aiResponseContent;
            }
            
            responseCard.style.display = "block";
            // Smooth scroll to view response
            responseCard.scrollIntoView({ behavior: 'smooth' });
        } else {
            alert(`AI Error: ${data.error || "Request failed"}`);
        }
    } catch (e) {
        alert("Failed to reach local server: " + e.message);
    } finally {
        panel.classList.remove("loading-pulse");
    }
}

// Discard AI changes
function handleDiscard() {
    const responseCard = document.getElementById("responseCard");
    responseCard.style.display = "none";
    document.getElementById("promptInput").value = "";
}

// Write selection back to active document
async function handleReplaceSelection() {
    if (!aiResponseContent) return;
    
    if (typeof Word === "undefined" || !Office.context || !Office.context.document) {
        alert("Word context not found. Cannot apply changes outside of Microsoft Word desktop/web app.");
        return;
    }
    
    try {
        await Word.run(async (context) => {
            const selection = context.document.getSelection();
            
            if (aiResponseFormat === "html") {
                selection.insertHtml(aiResponseContent, Word.InsertLocation.replace);
            } else {
                selection.insertText(aiResponseContent, Word.InsertLocation.replace);
            }
            
            await context.sync();
        });
        
        // Hide card and clear input
        handleDiscard();
    } catch (error) {
        alert("Error writing back to Word: " + error.message);
    }
}
