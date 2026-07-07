<script>
  import { onMount } from "svelte";
  import { channel } from "@elyra/runtime";
  import { api } from "./bindings";
  import { getTheme, applyTheme } from "./theme.js";
  import { marked } from "marked";

  const CADENCES = ["hourly", "daily", "weekly", "monthly", "manual"];
  const LANGUAGES = [
    { value: "English", label: "English" },
    { value: "Norwegian", label: "Norsk" },
  ];
  const TONES = [
    { value: "informative and friendly", label: "Informative and friendly" },
    { value: "lun og varm", label: "Lun og varm" },
    { value: "professional", label: "Professional" },
    { value: "casual and conversational", label: "Casual & conversational" },
    { value: "enthusiastic and energetic", label: "Enthusiastic" },
    { value: "formal", label: "Formal" },
    { value: "humorous", label: "Humorous" },
  ];
  const IMAGE_STYLES = [
    { value: "flat isometric", label: "Flat / isometric" },
    { value: "flat vector", label: "Flat vector" },
    { value: "risograph grain", label: "Risograph grain" },
    { value: "editorial collage", label: "Editorial collage" },
  ];
  const IMAGE_FORMATS = ["16:9", "4:3", "3:2", "1:1"];

  function blankSite() {
    return {
      id: 0,
      name: "",
      description: "",
      base_url: "",
      api_path: "/api/articles",
      token: "",
      themes: "",
      tone: "informative and friendly",
      language: "English",
      cadence: "daily",
      active: true,
      auto_publish: true,
      image_enabled: true,
      image_style: "flat isometric",
      image_format: "16:9",
      next_run_at: 0,
      created_at: 0,
      updated_at: 0,
    };
  }

  let sites = $state([]);
  let form = $state(null); // the site being edited (or a blank new one)
  let articles = $state([]);
  let preview = $state(null); // the article shown in the preview modal
  let editing = $state(false);
  let edit = $state({ title: "", excerpt: "", body: "", theme: "", category: "" });
  let busy = $state(false);
  let working = $state(""); // message shown during long operations
  let generating = $state(false); // spinner state for Generate now
  let error = $state("");

  // settings
  let showSettings = $state(false);
  let settings = $state({
    anthropic_key_set: false,
    model: "claude-sonnet-5",
    openai_key_set: false,
    image_model: "gpt-image-1",
  });
  let keyInput = $state("");
  let modelInput = $state("");
  let openaiKeyInput = $state("");
  let imageModelInput = $state("");

  // theme
  let theme = $state(getTheme());
  function cycleTheme() {
    theme = theme === "auto" ? "light" : theme === "light" ? "dark" : "auto";
    applyTheme(theme);
  }

  const fmt = (secs) => (secs ? new Date(secs * 1000).toLocaleString() : "—");

  async function loadSites() {
    sites = await api.list_sites();
    if (form && form.id !== 0 && !sites.some((s) => s.id === form.id)) form = null;
  }

  async function loadArticles() {
    articles = form && form.id !== 0 ? await api.list_articles(form.id) : [];
  }

  function selectSite(site) {
    error = "";
    form = { ...site };
    loadArticles();
  }

  function newSite() {
    error = "";
    form = blankSite();
    articles = [];
  }

  async function run(fn, workingMsg = "") {
    error = "";
    busy = true;
    working = workingMsg;
    try {
      return await fn();
    } catch (e) {
      error = String(e.message ?? e);
    } finally {
      busy = false;
      working = "";
    }
  }

  const saveSite = () =>
    run(async () => {
      const saved = await api.save_site(form);
      form = { ...saved };
      await loadSites();
      await loadArticles();
    });

  const deleteSite = () =>
    run(async () => {
      if (form.id === 0) return (form = null);
      await api.delete_site(form.id);
      form = null;
      await loadSites();
    });

  const generateNow = () =>
    run(async () => {
      generating = true;
      try {
        await api.generate_now(form.id);
        await loadArticles();
      } finally {
        generating = false;
      }
    }, "Jobber med artikkel … vent (tekst + bilde kan ta noen sekunder)");

  const publish = (id) => run(async () => { const a = await api.publish_article(id); if (preview && preview.id === id) preview = a; await loadArticles(); }, "Publiserer …");
  const removeArticle = (id) => run(async () => { await api.delete_article(id); if (preview && preview.id === id) preview = null; await loadArticles(); });

  const regenImage = (id) =>
    run(async () => {
      const a = await api.regenerate_image(id);
      if (preview && preview.id === id) preview = a;
      await loadArticles();
    }, "Genererer bilde … vent");

  function openPreview(article) { preview = article; editing = false; }
  function closePreview() { preview = null; editing = false; }
  function startEdit() {
    edit = {
      title: preview.title,
      excerpt: preview.excerpt,
      body: preview.body,
      theme: preview.theme,
      category: preview.category,
    };
    editing = true;
  }
  function cancelEdit() { editing = false; }
  const saveEdit = () =>
    run(async () => {
      const a = await api.update_article(preview.id, edit.title, edit.excerpt, edit.body, edit.theme, edit.category);
      preview = a;
      editing = false;
      await loadArticles();
    }, "Lagrer …");

  async function loadSettings() {
    settings = await api.get_settings();
    modelInput = settings.model;
    imageModelInput = settings.image_model;
  }
  const saveSettings = () =>
    run(async () => {
      await api.save_settings(keyInput, modelInput, openaiKeyInput, imageModelInput);
      keyInput = "";
      openaiKeyInput = "";
      await loadSettings();
      showSettings = false;
    });

  onMount(() => {
    loadSites();
    loadSettings();
    channel("sites").subscribe((id) => { if (id !== undefined) loadSites(); });
    channel("articles").subscribe((siteId) => {
      if (siteId !== undefined && form && siteId === form.id) loadArticles();
    });
  });
</script>

<div class="app">
  <header class="toolbar">
    <span class="brand"><img src="/icon.svg" width="22" height="22" alt="" style="vertical-align:-5px;border-radius:5px;margin-right:6px" />BlogWriter</span>
    <span class="spacer"></span>
    {#if !settings.anthropic_key_set}
      <span class="badge failed">no API key</span>
    {/if}
    <button class="btn" onclick={() => (showSettings = true)}>settings</button>
    <button class="btn" onclick={cycleTheme}>theme: {theme}</button>
  </header>

  {#if working}
    <div class="working-bar"><span class="dots">⏳</span> {working}</div>
  {/if}

  <div class="layout">
    <aside class="sidebar">
      <div class="section-label">Sites</div>
      {#each sites as site (site.id)}
        <button class="nav-item" class:active={form && form.id === site.id} onclick={() => selectSite(site)}>
          <span class="dot" class:on={site.active}></span>
          {site.name || "(unnamed)"}
          <span class="sub">{site.cadence}</span>
        </button>
      {/each}
      {#if sites.length === 0}
        <p class="muted" style="padding: 8px">No sites yet.</p>
      {/if}
      <button class="btn primary" style="margin-top: 8px" onclick={newSite}>+ New site</button>
    </aside>

    <main class="content" class:spin={busy}>
      {#if !form}
        <div class="card">
          <h2>Welcome to BlogWriter</h2>
          <p class="subtitle">
            Register your Laravel sites, set their themes and cadence, and BlogWriter
            will generate articles with Anthropic and publish them automatically.
          </p>
          <button class="btn primary" onclick={newSite}>+ New site</button>
        </div>
      {:else}
        <div class="page-head">
          <h2>{form.id === 0 ? "New site" : form.name || "(unnamed)"}</h2>
          {#if form.id !== 0}
            <div class="head-actions">
              <button class="btn" onclick={generateNow} disabled={generating}>
                {#if generating}<span class="spinner"></span> Genererer…{:else}Generate now{/if}
              </button>
              <button class="btn" onclick={deleteSite}>Delete</button>
            </div>
          {/if}
        </div>

        <div class="card" style="max-width: none">
          <div class="grid2">
            <div class="field"><label>Name</label><input bind:value={form.name} placeholder="My Blog" /></div>
            <div class="field"><label>Base URL</label><input bind:value={form.base_url} placeholder="https://blog.example.com" /></div>
            <div class="field"><label>API path</label><input bind:value={form.api_path} placeholder="/api/articles" /></div>
            <div class="field"><label>Bearer token</label><input bind:value={form.token} placeholder="Laravel API token" /></div>
            <div class="field">
              <label>Tone</label>
              <select bind:value={form.tone}>
                {#each TONES as t}<option value={t.value}>{t.label}</option>{/each}
              </select>
            </div>
            <div class="field">
              <label>Language</label>
              <select bind:value={form.language}>
                {#each LANGUAGES as l}<option value={l.value}>{l.label}</option>{/each}
              </select>
            </div>
            <div class="field">
              <label>Cadence</label>
              <select bind:value={form.cadence}>
                {#each CADENCES as c}<option value={c}>{c}</option>{/each}
              </select>
            </div>
            <div class="field" style="justify-content: flex-end; gap: 10px">
              <label class="check"><input type="checkbox" bind:checked={form.active} /> Active</label>
              <label class="check"><input type="checkbox" bind:checked={form.auto_publish} /> Auto-publish</label>
            </div>
            <div class="field">
              <label>Image style</label>
              <select bind:value={form.image_style}>
                {#each IMAGE_STYLES as s}<option value={s.value}>{s.label}</option>{/each}
              </select>
            </div>
            <div class="field">
              <label>Image format</label>
              <select bind:value={form.image_format}>
                {#each IMAGE_FORMATS as f}<option value={f}>{f}</option>{/each}
              </select>
            </div>
          </div>
          <div class="field" style="flex-direction: row; align-items: center; gap: 10px">
            <label class="check"><input type="checkbox" bind:checked={form.image_enabled} /> Generate cover image (Grove palette, web-optimized)</label>
          </div>
          <div class="field">
            <label>Site description (what the site is about, audience, goal)</label>
            <textarea bind:value={form.description} placeholder="E.g. A Norwegian consumer site helping people track and save on subscriptions, with a warm, practical voice."></textarea>
          </div>
          <div class="field">
            <label>Themes (comma-separated topics to rotate through)</label>
            <textarea bind:value={form.themes} placeholder="Laravel tips, PHP performance, developer productivity"></textarea>
          </div>
          <div class="form-actions">
            <button class="btn primary" onclick={saveSite}>{form.id === 0 ? "Create site" : "Save"}</button>
            {#if form.id !== 0}<span class="muted">next run: {fmt(form.next_run_at)}</span>{/if}
          </div>
          {#if error}<p class="err">{error}</p>{/if}
        </div>

        {#if form.id !== 0}
          <h2 style="margin: 22px 0 10px; font-size: 14px">Articles</h2>
          {#if articles.length === 0}
            <p class="muted">No articles yet — click <b>Generate now</b> or wait for the schedule.</p>
          {:else}
            <table>
              <thead><tr><th></th><th>Title</th><th>Theme</th><th>Status</th><th>Created</th><th></th></tr></thead>
              <tbody>
                {#each articles as a (a.id)}
                  <tr>
                    <td style="width: 108px">
                      {#if a.image}
                        <img src={a.image} alt="" style="width:96px;height:54px;object-fit:cover;border-radius:5px;border:1px solid var(--border);cursor:pointer" onclick={() => openPreview(a)} />
                      {/if}
                    </td>
                    <td>
                      <button class="linklike" onclick={() => openPreview(a)}>{a.title}</button>
                      <br /><span class="muted">{a.excerpt}</span>
                      {#if a.error}<br /><span class="err">{a.error}</span>{/if}
                    </td>
                    <td class="muted">{a.theme}{#if a.category && a.category !== a.theme}<br /><span style="opacity:0.7">⤷ {a.category}</span>{/if}</td>
                    <td><span class="badge {a.status}">{a.status}</span></td>
                    <td class="muted">{fmt(a.created_at)}</td>
                    <td style="white-space: nowrap">
                      <button class="btn" onclick={() => openPreview(a)}>Preview</button>
                      <button class="btn" onclick={() => publish(a.id)}>{a.status === "published" ? "Re-publish" : "Publish"}</button>
                      <button class="btn" onclick={() => removeArticle(a.id)}>✕</button>
                    </td>
                  </tr>
                {/each}
              </tbody>
            </table>
          {/if}
        {/if}
      {/if}
    </main>
  </div>
</div>

{#if showSettings}
  <div class="backdrop" onclick={(e) => e.target === e.currentTarget && (showSettings = false)}>
    <div class="modal">
      <h2>Settings</h2>
      <div class="field">
        <label>Anthropic API key {settings.anthropic_key_set ? "(set — leave blank to keep)" : ""}</label>
        <input type="password" bind:value={keyInput} placeholder="sk-ant-..." />
      </div>
      <div class="field">
        <label>Text model (Anthropic)</label>
        <input bind:value={modelInput} placeholder="claude-sonnet-5" />
      </div>
      <div class="field">
        <label>OpenAI API key (images) {settings.openai_key_set ? "(set — leave blank to keep)" : ""}</label>
        <input type="password" bind:value={openaiKeyInput} placeholder="sk-..." />
      </div>
      <div class="field">
        <label>Image model (OpenAI)</label>
        <input bind:value={imageModelInput} placeholder="gpt-image-1" />
      </div>
      <p class="muted">Text is written by Anthropic; cover images are generated by OpenAI (Claude can't make images).</p>
      <div class="form-actions">
        <button class="btn primary" onclick={saveSettings}>Save</button>
        <button class="btn" onclick={() => (showSettings = false)}>Cancel</button>
      </div>
      {#if error}<p class="err">{error}</p>{/if}
    </div>
  </div>
{/if}

{#if preview}
  <div class="backdrop" onclick={(e) => e.target === e.currentTarget && closePreview()}>
    <div class="modal preview">
      <div class="page-head">
        <h2>{editing ? "Rediger artikkel" : preview.title}</h2>
        <div class="head-actions">
          <span class="badge {preview.status}">{preview.status}</span>
          <button class="btn" onclick={closePreview}>Close</button>
        </div>
      </div>
      {#if preview.image}
        <img class="cover" src={preview.image} alt="" />
      {:else}
        <div class="cover placeholder">Ingen bilde — klikk “Generate image”</div>
      {/if}

      {#if editing}
        <div class="field"><label>Tittel</label><input bind:value={edit.title} /></div>
        <div class="field"><label>Ingress</label><textarea bind:value={edit.excerpt}></textarea></div>
        <div class="grid2">
          <div class="field"><label>Tema</label><input bind:value={edit.theme} /></div>
          <div class="field"><label>Kategori</label><input bind:value={edit.category} /></div>
        </div>
        <div class="field"><label>Brødtekst (Markdown)</label><textarea class="body-edit" bind:value={edit.body}></textarea></div>
        <div class="form-actions">
          <button class="btn primary" onclick={saveEdit}>Lagre</button>
          <button class="btn" onclick={cancelEdit}>Avbryt</button>
        </div>
      {:else}
        {#if preview.excerpt}<p class="subtitle">{preview.excerpt}</p>{/if}
        <div class="prose">{@html marked(preview.body || "")}</div>
        {#if preview.error}<p class="err">{preview.error}</p>{/if}
        <div class="form-actions">
          <button class="btn" onclick={startEdit}>Edit</button>
          <button class="btn primary" onclick={() => publish(preview.id)}>{preview.status === "published" ? "Re-publish" : "Publish"}</button>
          <button class="btn" onclick={() => regenImage(preview.id)}>{preview.image ? "Regenerate image" : "Generate image"}</button>
          <button class="btn" onclick={() => removeArticle(preview.id)}>Delete</button>
        </div>
      {/if}
    </div>
  </div>
{/if}
