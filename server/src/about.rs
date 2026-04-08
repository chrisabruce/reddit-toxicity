/// Returns the full about page HTML as a static string.
pub fn page() -> &'static str {
    r##"<!doctype html>
<html lang="en">
<head>
  <meta charset="utf-8">
  <meta name="viewport" content="width=device-width, initial-scale=1">
  <title>Reddit Toxicity Badge</title>
  <style>
    *,
    *::before,
    *::after {
      box-sizing: border-box;
      margin: 0;
    }

    :root {
      --bg: #111116;
      --surface: #1c1c22;
      --border: #2a2a32;
      --text: #e4e4e8;
      --muted: #8b8b96;
      --accent: #22c55e;
      --yellow: #eab308;
      --red: #ef4444;
      --mono: ui-monospace, "Cascadia Code", "Fira Code", Menlo, monospace;
      --sans: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, sans-serif;
    }

    html {
      background: var(--bg);
      color: var(--text);
      font-family: var(--sans);
      line-height: 1.6;
    }

    body {
      max-width: 42rem;
      margin: 0 auto;
      padding: 3rem 1.5rem 5rem;
    }

    h1 {
      font-size: 2rem;
      font-weight: 700;
      letter-spacing: -0.02em;
      margin-block-end: 0.25rem;
    }

    h2 {
      font-size: 1.25rem;
      font-weight: 600;
      margin-block-start: 2.5rem;
      margin-block-end: 0.75rem;
      color: var(--text);
    }

    h3 {
      font-size: 1rem;
      font-weight: 600;
      margin-block-start: 1.5rem;
      margin-block-end: 0.5rem;
    }

    p {
      margin-block-end: 1rem;
      color: var(--muted);
    }

    a {
      color: var(--accent);
      text-decoration: none;
    }

    a:hover {
      text-decoration: underline;
    }

    .subtitle {
      color: var(--muted);
      font-size: 1.1rem;
      margin-block-end: 2rem;
    }

    .try-it {
      background: var(--surface);
      border: 1px solid var(--border);
      border-radius: 0.5rem;
      padding: 1.25rem;
      margin-block: 1.5rem;
    }

    .try-it label {
      display: block;
      font-size: 0.8rem;
      color: var(--muted);
      text-transform: uppercase;
      letter-spacing: 0.05em;
      margin-block-end: 0.5rem;
    }

    .try-it-row {
      display: flex;
      gap: 0.5rem;
      align-items: stretch;
    }

    .try-it input {
      flex: 1;
      padding: 0.5rem 0.75rem;
      font-family: var(--mono);
      font-size: 0.9rem;
      background: var(--bg);
      color: var(--text);
      border: 1px solid var(--border);
      border-radius: 0.375rem;
    }

    .try-it input:focus {
      outline: 2px solid var(--accent);
      outline-offset: -1px;
    }

    .try-it select {
      padding: 0.5rem 0.5rem;
      font-family: var(--mono);
      font-size: 0.9rem;
      background: var(--bg);
      color: var(--text);
      border: 1px solid var(--border);
      border-radius: 0.375rem;
    }

    .try-it button {
      padding: 0.5rem 1.25rem;
      background: var(--accent);
      color: #000;
      font-weight: 600;
      font-size: 0.9rem;
      border: none;
      border-radius: 0.375rem;
      cursor: pointer;
    }

    .try-it button:hover {
      filter: brightness(1.1);
    }

    #badge-output {
      margin-block-start: 1rem;
      min-height: 3.5rem;
      display: flex;
      align-items: center;
    }

    #badge-output img {
      max-width: 100%;
      height: auto;
    }

    .embed-url {
      margin-block-start: 0.5rem;
      font-size: 0.8rem;
      color: var(--muted);
      word-break: break-all;
    }

    .embed-url code {
      cursor: pointer;
    }

    table {
      width: 100%;
      border-collapse: collapse;
      margin-block: 1rem;
      font-size: 0.9rem;
    }

    th {
      text-align: start;
      font-weight: 600;
      color: var(--text);
      padding: 0.5rem 0.75rem;
      border-block-end: 1px solid var(--border);
    }

    td {
      padding: 0.5rem 0.75rem;
      color: var(--muted);
      border-block-end: 1px solid var(--border);
    }

    tr:last-child td {
      border: none;
    }

    code {
      font-family: var(--mono);
      font-size: 0.85em;
      background: var(--surface);
      padding: 0.15em 0.4em;
      border-radius: 0.25rem;
    }

    pre {
      background: var(--surface);
      border: 1px solid var(--border);
      border-radius: 0.5rem;
      padding: 1rem;
      overflow-x: auto;
      margin-block: 1rem;
      font-size: 0.85rem;
      line-height: 1.5;
    }

    pre code {
      background: none;
      padding: 0;
    }

    .color-dot {
      display: inline-block;
      width: 0.65rem;
      height: 0.65rem;
      border-radius: 50%;
      vertical-align: middle;
      margin-inline-end: 0.35rem;
    }

    .note {
      background: var(--surface);
      border-inline-start: 3px solid var(--accent);
      padding: 0.75rem 1rem;
      margin-block: 1rem;
      border-radius: 0 0.375rem 0.375rem 0;
      font-size: 0.9rem;
    }

    footer {
      margin-block-start: 3rem;
      padding-block-start: 1.5rem;
      border-block-start: 1px solid var(--border);
      color: var(--muted);
      font-size: 0.85rem;
    }

    footer p {
      margin-block-end: 0.25rem;
    }
  </style>
</head>
<body>
  <header>
    <h1>Reddit Toxicity Badge</h1>
    <p class="subtitle">Real-time toxicity scoring for any subreddit, as an embeddable badge.</p>
  </header>

  <section class="try-it" aria-label="Try it">
    <label for="subreddit-input">Try a subreddit</label>
    <div class="try-it-row">
      <input type="text" id="subreddit-input" value="rust" placeholder="subreddit name" autocomplete="off" spellcheck="false">
      <select id="format-select" aria-label="Output format">
        <option value="svg" selected>.svg</option>
        <option value="png">.png</option>
        <option value="jpg">.jpg</option>
        <option value="html">.html</option>
      </select>
      <button type="button" id="go-btn">Go</button>
    </div>
    <output id="badge-output"></output>
    <p class="embed-url" id="embed-url"></p>
  </section>

  <h2>How it works</h2>
  <p>The badge scores community toxicity from <strong>0&ndash;100</strong> using only public vote data from Reddit&rsquo;s API &mdash; no text analysis, no AI, no sentiment models. Three signals are combined:</p>
  <table>
    <thead>
      <tr>
        <th scope="col">Signal</th>
        <th scope="col">Weight</th>
        <th scope="col">What it measures</th>
      </tr>
    </thead>
    <tbody>
      <tr>
        <td>New post upvote ratio</td>
        <td>55%</td>
        <td>Average <code>upvote_ratio</code> across up to 100 new posts. Uses <code>/new</code> instead of <code>/hot</code> to avoid survivorship bias.</td>
      </tr>
      <tr>
        <td>OP comment negativity</td>
        <td>30%</td>
        <td>Fraction of the original poster&rsquo;s own comments that are downvoted. Measures how hostile a community is toward the people posting.</td>
      </tr>
      <tr>
        <td>Negative comment %</td>
        <td>15%</td>
        <td>Fraction of all sampled comments with score&nbsp;&lt;&nbsp;2. Captures general comment section hostility.</td>
      </tr>
    </tbody>
  </table>
  <p>Posts with zero comments are excluded &mdash; they&rsquo;re too new to carry signal.</p>

  <h2>Score ranges</h2>
  <table>
    <thead>
      <tr>
        <th scope="col">Score</th>
        <th scope="col">Label</th>
      </tr>
    </thead>
    <tbody>
      <tr><td>0&ndash;20</td><td><span class="color-dot" style="background:#22c55e"></span>Very Low</td></tr>
      <tr><td>21&ndash;35</td><td><span class="color-dot" style="background:#22c55e"></span>Low</td></tr>
      <tr><td>36&ndash;50</td><td><span class="color-dot" style="background:#eab308"></span>Moderate</td></tr>
      <tr><td>51&ndash;65</td><td><span class="color-dot" style="background:#eab308"></span>High</td></tr>
      <tr><td>66&ndash;100</td><td><span class="color-dot" style="background:#ef4444"></span>Very High</td></tr>
    </tbody>
  </table>

  <h2>Embed a badge</h2>
  <p>Drop this into any HTML page, README, or forum post. Available as SVG, PNG, or JPEG:</p>
  <pre><code>&lt;img src="https://your-host/toxicity/r/subreddit.svg?size=420" alt="toxicity badge"&gt;</code></pre>

  <h3>Supported formats</h3>
  <table>
    <thead>
      <tr>
        <th scope="col">Extension</th>
        <th scope="col">Type</th>
        <th scope="col">Notes</th>
      </tr>
    </thead>
    <tbody>
      <tr><td><code>.svg</code></td><td>image/svg+xml</td><td>Default. Scalable, smallest size.</td></tr>
      <tr><td><code>.png</code></td><td>image/png</td><td>Rasterized with embedded font.</td></tr>
      <tr><td><code>.jpg</code> / <code>.jpeg</code></td><td>image/jpeg</td><td>White background, 90% quality.</td></tr>
      <tr><td><code>.html</code></td><td>text/html</td><td>Social card page with Open Graph tags for link unfurling.</td></tr>
    </tbody>
  </table>

  <h3>Query parameters</h3>
  <table>
    <thead>
      <tr>
        <th scope="col">Param</th>
        <th scope="col">Default</th>
        <th scope="col">Range</th>
        <th scope="col">Description</th>
      </tr>
    </thead>
    <tbody>
      <tr>
        <td><code>size</code></td>
        <td>420</td>
        <td>200&ndash;650</td>
        <td>Badge width in pixels</td>
      </tr>
    </tbody>
  </table>

  <h2>API</h2>
  <table>
    <thead>
      <tr>
        <th scope="col">Endpoint</th>
        <th scope="col">Description</th>
      </tr>
    </thead>
    <tbody>
      <tr>
        <td><code>/toxicity/r/{sub}.{ext}</code></td>
        <td>Badge for a subreddit (<code>.svg</code>, <code>.png</code>, <code>.jpg</code>, <code>.jpeg</code>, <code>.html</code>)</td>
      </tr>
      <tr>
        <td><code>/toxicity/{sub}.{ext}</code></td>
        <td>Same, without the <code>r/</code> prefix</td>
      </tr>
      <tr>
        <td><code>/health</code></td>
        <td>Returns <code>ok</code></td>
      </tr>
      <tr>
        <td><code>/about</code></td>
        <td>This page</td>
      </tr>
    </tbody>
  </table>

  <div class="note">
    <p>Scores are cached for up to 24 hours (configurable). The server works without Reddit OAuth credentials using the public API, but setting up an <a href="https://www.reddit.com/prefs/apps">OAuth app</a> gives you a dedicated rate limit.</p>
  </div>

  <footer>
    <p>Open source &mdash; <a href="https://github.com/chrisabruce/reddit-toxicity">chrisabruce/reddit-toxicity</a></p>
    <p>No text is analyzed. Only vote counts and ratios.</p>
  </footer>

  <script>
    const input = document.getElementById("subreddit-input");
    const formatSelect = document.getElementById("format-select");
    const btn = document.getElementById("go-btn");
    const output = document.getElementById("badge-output");
    const embedUrl = document.getElementById("embed-url");

    function loadBadge() {
      const sub = input.value.trim().replace(/^r\//, "");
      if (!sub) return;
      const ext = formatSelect.value;
      const encodedSub = encodeURIComponent(sub);
      const fullUrl = location.origin + "/toxicity/r/" + encodedSub + "." + ext;

      if (ext === "html") {
        output.innerHTML = "";
        const a = document.createElement("a");
        a.href = "/toxicity/r/" + encodedSub + ".html";
        a.target = "_blank";
        a.textContent = "Open social card for r/" + sub;
        output.appendChild(a);
        embedUrl.innerHTML = "Share: <code title='Click to copy'>" + fullUrl + "</code>";
        return;
      }

      const path = "/toxicity/r/" + encodedSub + "." + ext + "?size=500";
      output.innerHTML = "";
      const img = document.createElement("img");
      img.src = path;
      img.alt = "Toxicity badge for r/" + sub;
      output.appendChild(img);
      embedUrl.innerHTML = "Embed: <code title='Click to copy'>&lt;img src=\"" + fullUrl + "\"&gt;</code>";
    }

    btn.addEventListener("click", loadBadge);
    input.addEventListener("keydown", function(e) {
      if (e.key === "Enter") loadBadge();
    });
    formatSelect.addEventListener("change", loadBadge);

    embedUrl.addEventListener("click", function(e) {
      if (e.target.tagName === "CODE") {
        navigator.clipboard.writeText(e.target.textContent);
        e.target.style.outline = "2px solid var(--accent)";
        setTimeout(function() { e.target.style.outline = ""; }, 600);
      }
    });

    loadBadge();
  </script>
</body>
</html>"##
}
