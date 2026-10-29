'use strict';
const http = require('http');
const fs = require('fs');
const crypto = require('crypto');
const { spawn } = require('child_process');
const pty = require('node-pty');
const { WebSocketServer } = require('ws');
const config = require('./config');
process.chdir(__dirname);
const token = crypto.randomBytes(32).toString('hex');
let catalog = [];
let catalogError = '';
function refreshCatalog() {
  try { catalog = config.getCatalog(); catalogError = ''; }
  catch { catalog = []; catalogError = '无法读取 Pi 模型，请确认已安装 Pi 并配置供应商。'; }
  return catalog;
}
refreshCatalog();
const routes = { '/': ['index.html', 'text/html; charset=utf-8'], '/client.js': ['client.js', 'text/javascript; charset=utf-8'], '/style.css': ['style.css', 'text/css; charset=utf-8'], '/xterm.js': ['node_modules/@xterm/xterm/lib/xterm.js', 'text/javascript'], '/xterm.css': ['node_modules/@xterm/xterm/css/xterm.css', 'text/css'], '/fit.js': ['node_modules/@xterm/addon-fit/lib/addon-fit.js', 'text/javascript'] };
function json(res, status, data) { res.writeHead(status, { 'Content-Type': 'application/json; charset=utf-8', 'Cache-Control': 'no-store' }); res.end(JSON.stringify(data)); }
function readBody(req) { return new Promise((resolve, reject) => { let text = ''; req.on('data', chunk => { text += chunk; if (text.length > 8192) { reject(new Error('请求过大')); req.destroy(); } }); req.on('end', () => { try { resolve(JSON.parse(text)); } catch { reject(new Error('请求格式错误')); } }); req.on('error', reject); }); }
const server = http.createServer(async (req, res) => {
  const u = new URL(req.url, 'http://localhost');
  const route = routes[u.pathname];
  if (route && req.method === 'GET') {
    res.writeHead(200, { 'Content-Type': route[1], 'Cache-Control': 'no-store', 'X-Content-Type-Options': 'nosniff', 'Content-Security-Policy': "default-src 'self'; script-src 'self'; style-src 'self' 'unsafe-inline'; connect-src 'self'" });
    return fs.createReadStream(route[0]).pipe(res);
  }
  if (req.headers['x-workbench-token'] !== token) return json(res, 403, { error: '无权访问' });
  try {
    if (u.pathname === '/api/catalog' && req.method === 'GET') {
      refreshCatalog();
      return json(res, 200, { providers: catalog, error: catalogError, settings: config.loadSettings(), version: require('./package.json').version });
    }
    if (u.pathname === '/api/settings' && req.method === 'POST') {
      const body = await readBody(req);
      const settings = config.loadSettings();
      if ('workdir' in body) settings.workdir = config.validateWorkdir(body.workdir);
      if ('theme' in body) { if (!config.THEMES.includes(body.theme)) throw new Error('无效主题'); settings.theme = body.theme; }
      if ('thinking' in body) {
        const { provider, model, level } = body.thinking || {};
        const selected = catalog.find(p => p.id === provider)?.models.find(m => m.id === model);
        if (!selected || !selected.reasoning || !config.THINKING_LEVELS.includes(level)) throw new Error('无效的模型或思考程度');
        if (level === 'default') delete settings.thinking[`${provider}/${model}`];
        else settings.thinking[`${provider}/${model}`] = level;
      }
      config.saveSettings(settings);
      return json(res, 200, settings);
    }
    if (u.pathname === '/api/thinking-levels' && req.method === 'GET') {
      const providerId = u.searchParams.get('provider');
      const modelId = u.searchParams.get('model');
      let providers = catalog;
      if (providerId || modelId) {
        const provider = catalog.find(p => p.id === providerId);
        const model = provider?.models.find(m => m.id === modelId);
        if (!model) return json(res, 400, { error: '无效的模型' });
        providers = [{ ...provider, models: [model] }];
      }
      const settings = config.loadSettings();
      config.validateWorkdir(settings.workdir);
      return json(res, 200, await config.effectiveThinking(providers, settings));
    }
    if (u.pathname === '/api/quota' && req.method === 'GET') return json(res, 200, Object.fromEntries(await Promise.all(catalog.map(async p => [p.id, await config.quota(p)]))));
    return json(res, 404, { error: '未找到接口' });
  } catch (error) { return json(res, 400, { error: error.message || '操作失败' }); }
});
const wss = new WebSocketServer({ noServer: true });
server.on('upgrade', (req, socket, head) => {
  const u = new URL(req.url, 'http://localhost');
  const origin = `http://127.0.0.1:${server.address().port}`;
  const provider = catalog.find(x => x.id === u.searchParams.get('provider'));
  const model = provider?.models.find(x => x.id === u.searchParams.get('model'));
  if (req.headers.origin !== origin || u.pathname !== '/terminal' || u.searchParams.get('token') !== token || !model) return socket.destroy();
  wss.handleUpgrade(req, socket, head, ws => wss.emit('connection', ws, provider, model));
});
wss.on('connection', (ws, provider, model) => {
  let shell;
  try {
    const settings = config.loadSettings();
    const cwd = config.validateWorkdir(settings.workdir);
    const stored = settings.thinking[`${provider.id}/${model.id}`];
    const level = model.reasoning && config.THINKING_LEVELS.includes(stored) ? stored : 'default';
    const command = config.shellCommand(provider.id, model.id, level);
    shell = pty.spawn(command.shell, command.args, { name: 'xterm-256color', cols: 100, rows: 30, cwd, env: { ...process.env, PI_HARDWARE_CURSOR: '1' } });
  } catch (error) { ws.close(1011, '无法启动会话，请检查工作目录和 Pi 安装'); return; }
  shell.onData(data => { if (ws.readyState === 1) ws.send(data); });
  shell.onExit(() => ws.close());
  ws.on('message', raw => { try { const m = JSON.parse(raw); if (m.type === 'input' && typeof m.data === 'string') shell.write(m.data); if (m.type === 'resize' && Number.isInteger(m.cols) && Number.isInteger(m.rows) && m.cols > 0 && m.cols < 500 && m.rows > 0 && m.rows < 500) shell.resize(m.cols, m.rows); } catch {} });
  let closed = false;
  function stop() { if (!closed) { closed = true; try { shell.kill(); } catch {} } }
  ws.on('close', stop); ws.on('error', stop);
});
server.listen(0, '127.0.0.1', () => {
  const url = `http://127.0.0.1:${server.address().port}/#${token}`;
  console.log(url);
  if (process.argv.includes('--open')) {
    const command = process.platform === 'win32' ? ['rundll32.exe', ['url.dll,FileProtocolHandler', url]] : process.platform === 'darwin' ? ['open', [url]] : ['xdg-open', [url]];
    spawn(command[0], command[1], { detached: true, stdio: 'ignore' }).unref();
  }
});
