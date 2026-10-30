'use strict';
const fs = require('fs');
const path = require('path');
const os = require('os');
const { execFileSync, spawn } = require('child_process');

const THINKING_LEVELS = ['default', 'off', 'minimal', 'low', 'medium', 'high', 'xhigh', 'max'];
const THEMES = ['mint', 'jilan', 'dongqing', 'mist', 'neon', 'aurora'];
const LANGUAGES = ['zh-CN', 'en'];
const configDir = process.env.PI_CODING_AGENT_DIR || path.join(os.homedir(), '.pi', 'agent');
const settingsDir = process.env.PI_WORKBENCH_CONFIG_DIR || path.join(os.homedir(), '.pi-workbench');
const settingsFile = path.join(settingsDir, 'settings.json');
const legacySettings = path.join(__dirname, 'preferences.json');
const defaultSettings = () => ({ theme: 'mint', workdir: os.homedir(), thinking: {} });
function readJson(file, fallback = {}) { try { return JSON.parse(fs.readFileSync(file, 'utf8')); } catch { return fallback; } }
function loadSettings() {
  const raw = readJson(settingsFile, readJson(legacySettings));
  const settings = { theme: raw.theme, workdir: raw.workdir, thinking: raw.thinking, language: LANGUAGES.includes(raw.language) ? raw.language : null };
  if (!THEMES.includes(settings.theme)) settings.theme = 'mint';
  if (typeof settings.workdir !== 'string' || !path.isAbsolute(settings.workdir)) settings.workdir = os.homedir();
  if (!settings.thinking || typeof settings.thinking !== 'object' || Array.isArray(settings.thinking)) settings.thinking = {};
  return settings;
}
function saveSettings(settings) {
  fs.mkdirSync(settingsDir, { recursive: true, mode: 0o700 });
  const temp = `${settingsFile}.${process.pid}.tmp`;
  fs.writeFileSync(temp, JSON.stringify(settings, null, 2), { mode: 0o600 });
  fs.renameSync(temp, settingsFile);
}
function validateWorkdir(value) {
  if (typeof value !== 'string' || !path.isAbsolute(value)) throw new Error('请输入绝对路径');
  let stat; try { stat = fs.statSync(value); } catch { throw new Error('目录不存在或无法访问'); }
  if (!stat.isDirectory()) throw new Error('请选择目录');
  return path.resolve(value);
}
function piCommand(args, options = {}) {
  const common = { encoding: 'utf8', windowsHide: true, timeout: 15000, ...options };
  // Windows npm installs pi.cmd; arguments here are fixed by the application, except
  // provider IDs which are constrained to safe characters before invoking cmd.exe.
  if (process.platform === 'win32') {
    if (args.some(arg => !/^[\w.\/-]+$/.test(arg))) throw new Error('不支持的 Pi 参数');
    return execFileSync(process.env.ComSpec || 'cmd.exe', ['/d', '/s', '/c', ['pi.cmd', ...args].join(' ')], common);
  }
  return execFileSync('pi', args, common);
}
function parseModelList(text) {
  const groups = new Map();
  for (const line of text.split(/\r?\n/)) {
    const m = line.trim().match(/^(\S+)\s+(\S+)\s+\S+\s+\S+\s+(yes|no)\s+(yes|no)\s*$/i);
    if (!m) continue;
    const [, id, model, reasoning] = m;
    if (!groups.has(id)) groups.set(id, []);
    groups.get(id).push({ id: model, reasoning: reasoning.toLowerCase() === 'yes' });
  }
  return [...groups].map(([id, models]) => ({ id, label: id, models }));
}
function getCatalog() { return parseModelList(piCommand(['--list-models'])); }

function parseThinkingResponse(line) {
  try {
    const message = JSON.parse(line);
    const level = message.data?.thinkingLevel;
    return message.id === 'workbench-thinking' && message.success === true && THINKING_LEVELS.includes(level) && level !== 'default' ? level : null;
  } catch { return null; }
}
// Ask Pi itself: its startup logic merges project settings and clamps model capabilities.
// These probes do not send prompts, load extensions or create persisted sessions.
function probeThinking(provider, model, cwd, override) {
  return new Promise(resolve => {
    const args = ['--mode', 'rpc', '--provider', provider, '--model', model, '--no-session', '--no-extensions', '--no-skills', '--no-mcp', '--offline'];
    if (override && override !== 'default') args.push('--thinking', override);
    if (args.some(arg => !/^[\w.\/:@+-]+$/.test(arg))) return resolve(null);
    const executable = process.platform === 'win32' ? (process.env.ComSpec || 'cmd.exe') : 'pi';
    const childArgs = process.platform === 'win32' ? ['/d', '/s', '/c', ['pi.cmd', ...args].join(' ')] : args;
    let child;
    try { child = spawn(executable, childArgs, { cwd, windowsHide: true, stdio: ['pipe', 'pipe', 'ignore'] }); }
    catch { return resolve(null); }
    let output = '', done = false;
    const finish = value => { if (done) return; done = true; clearTimeout(timer); if (!child.killed) child.kill(); resolve(value); };
    const timer = setTimeout(() => finish(null), 12000);
    child.on('error', () => finish(null));
    child.stdin.on('error', () => finish(null));
    child.stdout.on('data', chunk => {
      output += chunk.toString('utf8');
      if (output.length > 1048576) return finish(null);
      let index;
      while ((index = output.indexOf('\n')) !== -1) {
        const line = output.slice(0, index); output = output.slice(index + 1);
        const level = parseThinkingResponse(line);
        if (level) return finish(level);
      }
    });
    child.on('exit', () => finish(null));
    child.stdin.end(JSON.stringify({ id: 'workbench-thinking', type: 'get_state' }) + '\n');
  });
}
async function effectiveThinking(providers, settings) {
  const tasks = [];
  for (const provider of providers) for (const model of provider.models) {
    if (model.reasoning) tasks.push({ provider: provider.id, model: model.id });
  }
  const result = {};
  let next = 0;
  async function worker() {
    while (next < tasks.length) {
      const { provider, model } = tasks[next++];
      const id = provider + '/' + model;
      result[id] = await probeThinking(provider, model, settings.workdir, settings.thinking[id]);
    }
  }
  await Promise.all(Array.from({ length: Math.min(4, tasks.length) }, worker));
  return result;
}

function getKey(provider) {
  // Pi resolves environment references and its credential store itself. No key is logged or sent to the browser.
  try { return piCommand(['auth', 'print-api-key', '--provider', provider], { stdio: ['ignore', 'pipe', 'ignore'] }).trim(); } catch { return ''; }
}
function quotaKind(id) {
  const custom = readJson(path.join(configDir, 'models.json')).providers?.[id];
  if (custom?.baseUrl) {
    try {
      const host = new URL(custom.baseUrl).hostname.toLowerCase();
      if (host === 'open.bigmodel.cn') return 'zhipu-cn';
      if (host === 'api.z.ai') return 'zhipu-global';
      if (['api.minimaxi.com', 'www.minimaxi.com'].includes(host)) return 'minimax-cn';
      if (host === 'api.minimax.io') return 'minimax-global';
      if (host === 'api.deepseek.com') return 'deepseek';
      return null;
    } catch { return null; }
  }
  return ({ 'minimax-cn': 'minimax-cn', 'minimax': 'minimax-global', 'zai-coding-cn': 'zhipu-cn', 'zai': 'zhipu-global', 'zai-coding': 'zhipu-global', 'deepseek': 'deepseek' })[id] || null;
}
async function quota(provider) {
  const kind = quotaKind(provider.id);
  if (!kind) return { error: '该供应商暂不支持额度查询' };
  const key = getKey(provider.id);
  if (!key) return { error: 'Pi 未提供可用于额度查询的 API Key' };
  let url, authorization;
  if (kind.startsWith('minimax')) { url = `https://${kind === 'minimax-cn' ? 'api.minimaxi.com' : 'api.minimax.io'}/v1/api/openplatform/coding_plan/remains`; authorization = `Bearer ${key}`; }
  else if (kind.startsWith('zhipu')) { url = `https://${kind === 'zhipu-cn' ? 'open.bigmodel.cn' : 'api.z.ai'}/api/monitor/usage/quota/limit`; authorization = key; }
  else { url = 'https://api.deepseek.com/user/balance'; authorization = `Bearer ${key}`; }
  try {
    const response = await fetch(url, { headers: { Authorization: authorization, Accept: 'application/json' }, signal: AbortSignal.timeout(12000) });
    if (!response.ok) return { error: `接口返回 HTTP ${response.status}` };
    const body = await response.json();
    if (kind === 'deepseek') return body.balance_infos?.length ? { balance: body.balance_infos.map(x => ({ currency: x.currency, total: x.total_balance })) } : { error: '未返回余额' };
    if (kind.startsWith('minimax')) {
      const item = body.model_remains?.find(x => x.model_name === 'general');
      if (!item) return { error: '接口未返回套餐额度' };
      const tiers = [];
      if (typeof item.current_interval_remaining_percent === 'number') tiers.push({ name: '5 小时', used: 100 - item.current_interval_remaining_percent, reset: item.end_time });
      if (item.current_weekly_status === 1 && typeof item.current_weekly_remaining_percent === 'number') tiers.push({ name: '本周', used: 100 - item.current_weekly_remaining_percent, reset: item.weekly_end_time });
      return tiers.length ? { tiers } : { error: '接口未返回额度百分比' };
    }
    const names = { 3: '5 小时', 6: '本周', 7: '本月' };
    const tiers = (body.data?.limits || []).filter(x => ['TOKENS_LIMIT', 'CREDIT_LIMIT'].includes(x.type) && typeof x.percentage === 'number').map(x => ({ name: names[x.unit] || `周期 ${x.unit}`, used: x.percentage, reset: x.nextResetTime }));
    return tiers.length ? { tiers } : { error: '接口未返回套餐额度' };
  } catch (error) { return { error: error.name === 'TimeoutError' ? '连接超时' : '网络或配置异常' }; }
}
function quoteShell(value) { return `'${String(value).replace(/'/g, "'\\''")}'`; }
function quotePowerShell(value) { return `'${String(value).replace(/'/g, "''")}'`; }
function shellCommand(provider, model, thinking, platform = process.platform) {
  const args = ['pi', '--provider', provider, '--model', model];
  if (thinking !== 'default') args.push('--thinking', thinking);
  if (platform === 'win32') return { shell: 'powershell.exe', args: ['-NoLogo', '-NoExit', '-Command', `& ${args.map(quotePowerShell).join(' ')}`] };
  return { shell: '/bin/sh', args: ['-c', `exec ${args.map(quoteShell).join(' ')}`] };
}
module.exports = { THINKING_LEVELS, THEMES, LANGUAGES, configDir, settingsDir, loadSettings, saveSettings, validateWorkdir, parseModelList, getCatalog, quota, shellCommand, probeThinking, effectiveThinking, parseThinkingResponse };
