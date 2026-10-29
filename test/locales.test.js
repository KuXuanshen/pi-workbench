'use strict';
const test = require('node:test');
const assert = require('node:assert/strict');
const fs = require('node:fs');
const os = require('node:os');
const path = require('node:path');
const { spawnSync } = require('node:child_process');
const zh = require('../locales/zh-CN.json');
const en = require('../locales/en.json');

test('语言文件包含相同的文案和占位符', () => {
  assert.deepEqual(Object.keys(en).sort(), Object.keys(zh).sort());
  const placeholders = text => [...text.matchAll(/\{\w+\}/g)].map(match => match[0]).sort();
  for (const key of Object.keys(zh)) {
    assert.ok(zh[key] && en[key], key);
    assert.deepEqual(placeholders(en[key]), placeholders(zh[key]), key);
  }
});

test('旧设置首次选择语言后持久化并保留原有配置', () => {
  const dir = fs.mkdtempSync(path.join(os.tmpdir(), 'pi-workbench-language-test-'));
  try {
    fs.writeFileSync(path.join(dir, 'settings.json'), JSON.stringify({ theme: 'mist', workdir: os.homedir(), thinking: { 'p/m': 'high' } }));
    const script = `const assert=require('node:assert/strict'), c=require('./config');let s=c.loadSettings();assert.equal(s.language,null);s.language='en';c.saveSettings(s);s=c.loadSettings();assert.equal(s.language,'en');assert.equal(s.theme,'mist');assert.equal(s.thinking['p/m'],'high');`;
    const result = spawnSync(process.execPath, ['-e', script], { cwd: path.join(__dirname, '..'), env: { ...process.env, PI_WORKBENCH_CONFIG_DIR: dir }, encoding: 'utf8' });
    assert.equal(result.status, 0, result.stderr);
  } finally { fs.rmSync(dir, { recursive: true, force: true }); }
});
