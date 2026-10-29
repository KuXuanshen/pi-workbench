'use strict';
const test = require('node:test');
const assert = require('node:assert/strict');
const path = require('node:path');
const config = require('../config');

test('从 Pi 模型清单解析真实供应商、模型和思考能力', () => {
  const list = 'provider       model       context max-out thinking images\nalpha          model-one   1M      128K    yes      no\nother          plain       128K    8K      no       yes\nalpha          model-two   1M      128K    yes      yes\n';
  assert.deepEqual(config.parseModelList(list), [
    { id: 'alpha', label: 'alpha', models: [{ id: 'model-one', reasoning: true }, { id: 'model-two', reasoning: true }] },
    { id: 'other', label: 'other', models: [{ id: 'plain', reasoning: false }] }
  ]);
});
test('仅允许已有的绝对工作目录', () => {
  assert.equal(config.validateWorkdir(__dirname), path.resolve(__dirname));
  assert.throws(() => config.validateWorkdir('relative-path'), /绝对路径/);
  assert.throws(() => config.validateWorkdir(path.join(__dirname, 'no-such-directory')), /不存在/);
  assert.throws(() => config.validateWorkdir(__filename), /目录/);
});
test('为各平台构造隔离参数，默认思考不覆盖 Pi', () => {
  const windows = config.shellCommand("provider'quoted", 'model', 'high', 'win32');
  assert.match(windows.args.at(-1), /'provider''quoted'/);
  assert.match(windows.args.at(-1), /'--thinking' 'high'/);
  const unix = config.shellCommand("provider'quoted", 'model', 'default', 'linux');
  assert.ok(unix.args.at(-1).includes("'provider'\\''quoted'"));
  assert.ok(!unix.args.at(-1).includes('--thinking'));
});

test('只接受 Pi RPC 的对应成功响应与有效思考程度', () => {
  const reply = (id, success, level) => JSON.stringify({ id, success, data: { thinkingLevel: level } });
  assert.equal(config.parseThinkingResponse(reply('workbench-thinking', true, 'high')), 'high');
  assert.equal(config.parseThinkingResponse(reply('other', true, 'high')), null);
  assert.equal(config.parseThinkingResponse(reply('workbench-thinking', false, 'low')), null);
  assert.equal(config.parseThinkingResponse(reply('workbench-thinking', true, 'default')), null);
  assert.equal(config.parseThinkingResponse('not-json'), null);
});
