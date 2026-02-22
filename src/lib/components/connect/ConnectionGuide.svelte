<script lang="ts">
  import { serverUrl, serverPort, authToken } from '$lib/stores/server-url.js';

  let copiedCurl = $state(false);
  let copiedPostman = $state(false);

  function curlExample(): string {
    return `curl -H "Authorization: Bearer ${$authToken}" ${$serverUrl}/api/events`;
  }

  function postmanExample(): string {
    return JSON.stringify({
      info: { name: 'Sermon Helper API', schema: 'https://schema.getpostman.com/json/collection/v2.1.0/collection.json' },
      item: [
        {
          name: 'List Events',
          request: {
            method: 'GET',
            url: `${$serverUrl}/api/events`,
            header: [{ key: 'Authorization', value: `Bearer ${$authToken}` }],
          },
        },
        {
          name: 'Create Event',
          request: {
            method: 'POST',
            url: `${$serverUrl}/api/events`,
            header: [
              { key: 'Authorization', value: `Bearer ${$authToken}` },
              { key: 'Content-Type', value: 'application/json' },
            ],
            body: {
              mode: 'raw',
              raw: JSON.stringify({ title: 'Sunday Service', date_time: new Date().toISOString(), speaker: 'Pastor' }, null, 2),
            },
          },
        },
        {
          name: 'WebSocket',
          request: {
            method: 'GET',
            url: `ws://localhost:${$serverPort}/ws?token=${$authToken}`,
          },
        },
      ],
    }, null, 2);
  }

  async function copyCurl() {
    await navigator.clipboard.writeText(curlExample());
    copiedCurl = true;
    setTimeout(() => { copiedCurl = false; }, 2000);
  }

  async function copyPostman() {
    await navigator.clipboard.writeText(postmanExample());
    copiedPostman = true;
    setTimeout(() => { copiedPostman = false; }, 2000);
  }
</script>

<div class="guide">
  <h3>API Examples</h3>

  <section>
    <h4>curl</h4>
    <pre><code>{curlExample()}</code></pre>
    <button onclick={copyCurl}>{copiedCurl ? 'Copied!' : 'Copy'}</button>
  </section>

  <section>
    <h4>WebSocket (connect from any client)</h4>
    <pre><code>ws://localhost:{$serverPort}/ws?token={$authToken}</code></pre>
  </section>

  <section>
    <h4>Postman Collection</h4>
    <button onclick={copyPostman}>{copiedPostman ? 'Copied!' : 'Copy Postman Collection JSON'}</button>
  </section>

  <section>
    <h4>Endpoint Reference</h4>
    <table>
      <thead>
        <tr>
          <th scope="col">Method</th>
          <th scope="col">Path</th>
          <th scope="col">Description</th>
        </tr>
      </thead>
      <tbody>
        <tr><td>GET</td><td>/api/events</td><td>List all events</td></tr>
        <tr><td>GET</td><td>/api/events/:id</td><td>Get event details</td></tr>
        <tr><td>POST</td><td>/api/events</td><td>Create event</td></tr>
        <tr><td>GET</td><td>/api/events/:id/recordings</td><td>List recordings</td></tr>
        <tr><td>POST</td><td>/api/events/:id/recordings</td><td>Add recording</td></tr>
        <tr><td>GET</td><td>/ws?token=…</td><td>WebSocket stream</td></tr>
      </tbody>
    </table>
  </section>
</div>

<style>
  .guide { max-width: 700px; }

  .guide h3 { margin: 0 0 1.5rem; }

  section { margin-bottom: 1.5rem; }

  h4 { margin: 0 0 0.5rem; font-size: 0.875rem; text-transform: uppercase; letter-spacing: 0.05em; color: #6b7280; }

  pre {
    padding: 0.75rem;
    background: #f3f4f6;
    border-radius: 0.375rem;
    overflow-x: auto;
    font-size: 0.875rem;
    margin-bottom: 0.5rem;
  }

  button {
    padding: 0.5rem 1rem;
    border: 1px solid #d1d5db;
    border-radius: 0.375rem;
    background: white;
    cursor: pointer;
    font-size: 0.875rem;
  }

  button:hover { background: #f3f4f6; }

  table { width: 100%; border-collapse: collapse; font-size: 0.875rem; }

  th, td { padding: 0.5rem; text-align: left; border-bottom: 1px solid #e5e7eb; }

  th { font-weight: 600; }
</style>
