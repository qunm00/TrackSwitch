const invoke = window.__TAURI__.core.invoke;

function setStatus(message) {
  document.getElementById("status").textContent = message;
}

async function loadPeer() {
  try {
    const name = await invoke("discover_peer");
    if (name) {
      const input = document.getElementById("peer");
      if (!input.value) {
        input.value = name;
      }
    }
  } catch (_e) {
    // Best-effort discovery. The user can type the name.
  }
}

async function loadDevices() {
  try {
    const devices = await invoke("list_devices");
    const select = document.getElementById("device");
    select.innerHTML = "";
    for (const d of devices) {
      const option = document.createElement("option");
      option.value = d.address;
      option.textContent = (d.name || d.address) + (d.connected ? " (connected)" : "");
      select.appendChild(option);
    }
    const trackpad = devices.find(
      (d) => d.name?.toLowerCase().includes("trackpad")
    );
    if (trackpad) {
      select.value = trackpad.address;
    }
  } catch (e) {
    setStatus(`Failed to list devices: ${e}`);
  }
}

async function save() {
  const trackpadAddress = document.getElementById("device").value;
  const peerName = document.getElementById("peer").value.trim();
  try {
    await invoke("set_config", { trackpadAddress, peerName });
    setStatus("Saved.");
  } catch (e) {
    setStatus(`Save failed: ${e}`);
  }
}

document.getElementById("save").addEventListener("click", save);
loadDevices();
loadPeer();
