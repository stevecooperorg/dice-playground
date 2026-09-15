const { test } = require('node:test');
const assert = require('node:assert/strict');
const { readFileSync } = require('node:fs');
const { runInNewContext } = require('node:vm');
const source = readFileSync('tutorial-static/open-document.js', 'utf8');

async function open({ content = '# Full lesson\n```dice\noutput("d6", d(6))\n```', ok = true, storageFails = false, href = 'https://dice.example/tutorial/pilot.dice' } = {}) {
  let listener, stored, destination;
  const status = { textContent: '' };
  const link = { href, parentElement: { querySelector: () => status } };
  runInNewContext(source, {
    URL, TextEncoder,
    document: { addEventListener: (_, handler) => { listener = handler; } },
    location: { href: 'https://dice.example/tutorial/pilot.html', origin: 'https://dice.example', assign: (url) => { destination = url; } },
    fetch: async () => ({ ok, status: 404, text: async () => content }),
    localStorage: { setItem: (key, value) => { if (storageFails) throw Error('Storage unavailable'); stored = [key, JSON.parse(value)]; } },
  });
  await listener({ target: { closest: () => link }, preventDefault() {} });
  return { stored, destination, status: status.textContent };
}

test('whole source beyond inline URL limit retains filename and Unicode', async () => {
  const content = '# Lesson 🎲\n' + 'Long prose. '.repeat(1000);
  const result = await open({ content });
  assert.deepEqual(result.stored, ['dice_playground_pending_load', { content, filename: 'pilot.dice' }]);
  assert.equal(result.destination, '/');
});

for (const options of [{ ok: false }, { storageFails: true }, { href: 'https://other.example/pilot.dice' }, { content: 'x'.repeat(64 * 1024 + 1) }]) {
  test(`handoff failure is visible: ${JSON.stringify(options).slice(0, 80)}`, async () => {
    const result = await open(options);
    assert.equal(result.destination, undefined);
    assert.match(result.status, /Could not open:/);
    assert.match(result.status, /Download the source/);
  });
}
