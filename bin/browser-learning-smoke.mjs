#!/usr/bin/env node
// Real Chromium/WASM smoke test using CDP and Node builtins (no browser package).
// Usage: CHROME=/path/to/chrome node bin/browser-learning-smoke.mjs /built/site
import { spawn } from 'node:child_process';
import { createServer } from 'node:http';
import { mkdtemp, readFile, rm, stat } from 'node:fs/promises';
import { tmpdir } from 'node:os';
import path from 'node:path';
import assert from 'node:assert/strict';

const root = path.resolve(process.argv[2] || 'dist');
const chrome = process.env.CHROME || '/Applications/Google Chrome.app/Contents/MacOS/Google Chrome';
const profile = await mkdtemp(path.join(tmpdir(), 'dice-browser-'));
const results = [];
const budgetMs = Number(process.env.MAX_LEARNING_MS || 5000);
assert.ok(Number.isFinite(budgetMs) && budgetMs > 0, 'positive browser smoke budget');
const longSource = '# Long document\n\n' + 'Source prose preserved. '.repeat(3500) + '\n\n```dice\noutput("Long source result", d(6))\n```\n';
const server = createServer(async (request, response) => {
  try {
    const pathname = new URL(request.url, 'http://localhost').pathname;
    if (pathname === '/tutorial/smoke-long.dice') {
      response.end(longSource); return;
    }
    if (pathname === '/tutorial/smoke-long.html') {
      response.setHeader('Content-Type', 'text/html');
      response.end('<main><p><a class="open-dice-document" href="smoke-long.dice">Open</a><span role="status"></span></p></main><script src="open-document.js"></script>'); return;
    }
    const filename = path.resolve(root, '.' + decodeURIComponent(pathname === '/' ? '/index.html' : pathname));
    if (!filename.startsWith(root + path.sep)) throw new Error('Invalid path');
    const mime = { '.html': 'text/html', '.js': 'text/javascript', '.wasm': 'application/wasm', '.css': 'text/css' }[path.extname(filename)] || 'text/plain';
    response.setHeader('Content-Type', mime);
    response.end(await readFile(filename));
  } catch { response.writeHead(404); response.end('Not found'); }
});
await new Promise(resolve => server.listen(0, '127.0.0.1', resolve));
const origin = `http://127.0.0.1:${server.address().port}`;
const child = spawn(chrome, ['--headless=new', '--no-first-run', '--no-default-browser-check', '--remote-debugging-port=0', `--user-data-dir=${profile}`, 'about:blank'], { stdio: 'ignore' });
child.on('error', error => console.error(error.message));
let socket;
const delay = ms => new Promise(resolve => setTimeout(resolve, ms));
async function until(fn, description, timeout = 30000) {
  const start = Date.now();
  while (Date.now() - start < timeout) {
    if (await fn()) return;
    await delay(50);
  }
  throw new Error(`Timed out: ${description}`);
}
try {
  await until(async () => { try { await stat(path.join(profile, 'DevToolsActivePort')); return true; } catch { return false; } }, 'Chromium startup');
  const port = (await readFile(path.join(profile, 'DevToolsActivePort'), 'utf8')).split('\n')[0];
  const target = await (await fetch(`http://127.0.0.1:${port}/json/new?about:blank`, { method: 'PUT' })).json();
  socket = new WebSocket(target.webSocketDebuggerUrl);
  await new Promise((resolve, reject) => { socket.onopen = resolve; socket.onerror = reject; });
  let nextId = 0;
  const pending = new Map();
  socket.onmessage = event => {
    const message = JSON.parse(event.data);
    const receiver = pending.get(message.id);
    if (!receiver) return;
    pending.delete(message.id);
    clearTimeout(receiver.timeout);
    if (message.error) receiver.reject(new Error(JSON.stringify(message.error)));
    else receiver.resolve(message.result);
  };
  function send(method, params = {}) {
    return new Promise((resolve, reject) => {
      const id = ++nextId;
      const timeout = setTimeout(() => { pending.delete(id); reject(new Error(`CDP timeout: ${method}`)); }, 30000);
      pending.set(id, { resolve, reject, timeout });
      socket.send(JSON.stringify({ id, method, params }));
    });
  }
  async function evaluate(expression) {
    const result = await send('Runtime.evaluate', { expression, returnByValue: true, awaitPromise: true });
    if (result.exceptionDetails) throw new Error(JSON.stringify(result.exceptionDetails));
    return result.result.value;
  }
  await send('Page.enable');
  async function navigate(relative) {
    await send('Page.navigate', { url: origin + relative });
    await until(async () => {
      try { return await evaluate(`location.pathname === ${JSON.stringify(relative)} && document.readyState === 'complete'`); }
      catch { return false; }
    }, `navigate ${relative}`);
  }
  async function openDocument(relative, source) {
    await navigate(relative);
    assert.ok(await evaluate('!!document.querySelector("a.open-dice-document")'));
    await evaluate('document.querySelector("a.open-dice-document").click()');
    await until(async () => {
      try { return await evaluate('location.pathname === "/" && !!document.querySelector("textarea")'); }
      catch { return false; }
    }, 'WASM editor load');
    assert.equal(await evaluate('document.querySelector("textarea").value'), source, 'whole source round-trip');
  }
  async function runAndCheck(label) {
    const start = Date.now();
    await evaluate('document.querySelector("button[title^=Run]").click()');
    await until(() => evaluate(`!![...document.querySelectorAll('[data-dice-output]')].find(e => e.getAttribute('data-dice-output') === ${JSON.stringify(label)})`), `report ${label}`);
    return Date.now() - start;
  }
  for (const [file, label] of [
    ['08-comparison-loops', 'Success by bonus at the chosen DC'],
    ['23-shared-pool-rule', 'Blades action categories'],
    ['25-damage-dependent-save', 'One Blood Elk hit'],
    ['30-after-roll-decisions', 'One-invocation policy'],
    ['31-report-labels', 'Clean theory comparison'],
    ['32-build-and-defend', 'One-target Fireball risk'],
  ]) {
    const source = await readFile(path.join(root, 'tutorial', file + '.dice'), 'utf8');
    await openDocument(`/tutorial/${file}.html`, source);
    assert.equal(await evaluate('document.querySelector("button[title=Files]").textContent.trim()'), file + '.dice');
    const elapsedMs = await runAndCheck(label);
    results.push({ file, elapsedMs, sourceRoundTrip: true });
    if (file === '23-shared-pool-rule') {
      const edited = source.replace('dice = 2', 'dice = 5').replaceAll('Blades action categories', 'Five-dice boundary');
      await evaluate(`document.querySelector('textarea').value = ${JSON.stringify(edited)}; document.querySelector('textarea').dispatchEvent(new Event('input', {bubbles:true}))`);
      results.push({ file: 'Blades callback at supported maximum five dice', elapsedMs: await runAndCheck('Five-dice boundary'), editedRerun: true });
    }
  }
  const theorizeRecipe = await readFile(path.join(root, 'cookbook/brindlewood-bay-theorize.dice'), 'utf8');
  await openDocument('/cookbook/brindlewood-bay-theorize.html', theorizeRecipe);
  results.push({ file: 'Brindlewood supplied-text recipe', elapsedMs: await runAndCheck('Initial Theorize categories'), sourceRoundTrip: true });
  const bladesRecipe = await readFile(path.join(root, 'cookbook/blades-in-the-dark.dice'), 'utf8');
  await openDocument('/cookbook/blades-in-the-dark.html', bladesRecipe);
  const largePool = bladesRecipe.replace('dice = 2', 'dice = 20').replaceAll('Blades action categories', 'Twenty-dice decomposition');
  await evaluate(`document.querySelector('textarea').value = ${JSON.stringify(largePool)}; document.querySelector('textarea').dispatchEvent(new Event('input', {bubbles:true}))`);
  results.push({ file: 'Blades decomposition at twenty dice', elapsedMs: await runAndCheck('Twenty-dice decomposition'), editedRerun: true });
  await openDocument('/tutorial/smoke-long.html', longSource);
  results.push({ file: 'Long source beyond URL payload limit', bytes: Buffer.byteLength(longSource), elapsedMs: await runAndCheck('Long source result'), sourceRoundTrip: true });
  await navigate('/tutorial/smoke-long.html');
  await evaluate('Storage.prototype.setItem = () => { throw new Error("Storage disabled in smoke test") }; document.querySelector("a.open-dice-document").click()');
  await until(() => evaluate('document.querySelector("[role=status]").textContent.includes("Download the source")'), 'storage error download fallback');
  results.push({ storageFailureFallback: true });
  await navigate('/tutorial/05-dice-notation.html');
  assert.ok(await evaluate('document.body.textContent.includes("Lesson moved")'));
  assert.ok(await evaluate('[...document.querySelectorAll("a")].some(a => a.href.includes("13-keep-and-drop.html"))'));
  results.push({ oldUrlMigration: true });
  for (const result of results) {
    if (result.elapsedMs !== undefined) assert.ok(result.elapsedMs <= budgetMs, `${result.file}: ${result.elapsedMs} ms exceeds ${budgetMs} ms smoke budget`);
  }
  console.log(JSON.stringify({ browser: 'Chromium headless; debug WASM', budgetMs, timing: 'wall-clock click to rendered report, includes automation overhead', results }, null, 2));
} finally {
  if (socket) socket.close();
  child.kill('SIGTERM');
  await new Promise(resolve => server.close(resolve));
  await delay(300);
  await rm(profile, { recursive: true, force: true, maxRetries: 5, retryDelay: 200 });
}
