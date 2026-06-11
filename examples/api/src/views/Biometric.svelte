<script>
  import { authenticate, checkStatus } from "@tauri-apps/plugin-biometric";

  export let onMessage;
  let allowDeviceCredential = true;
  let allowWatch = false;

  function status() {
    checkStatus().then(onMessage).catch(onMessage);
  }

  function auth() {
    authenticate("Tauri API wants to show it is awesome :)", {
      allowDeviceCredential,
      allowWatch,
      cancelTitle: "Cancel request",
      fallbackTitle: "Trying the fallback option",
      title: "Tauri API Auth",
      subtitle: "Please authenticate :)",
      confirmationRequired: false,
      maxAttemps: 1,
    })
      .then(onMessage)
      .catch(onMessage);
  }
</script>

<div>
  <input
    type="checkbox"
    id="allowDeviceCredential"
    bind:checked={allowDeviceCredential}
  />
  <label for="allowDeviceCredential">Allow device credential</label>
</div>
<div>
  <input type="checkbox" id="allowWatch" bind:checked={allowWatch} />
  <label for="allowWatch">Allow Apple Watch (macOS)</label>
</div>
<button class="btn" id="check-status" on:click={status}> Check status </button>
<button class="btn" id="cli-matches" on:click={auth}> Authenticate </button>
