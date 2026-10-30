'use strict';
const {test} = require('node:test');
const assert = require('node:assert/strict');
const fs = require('node:fs');
const path = require('node:path');
const {JSDOM} = require('jsdom');
const source = fs.readFileSync(path.join(__dirname, '../assets/ui-controls.js'), 'utf8');

function setup() {
  const dom = new JSDOM(`<!doctype html><body>
    <button id="sidebar-toggle" aria-label="收起侧边栏">☰</button>
    <label class="theme-picker">主题<select id="theme-select" aria-label="选择主题"><option value="mint">夜幕薄荷</option><option value="mist">雾蓝浅色</option></select></label>
    <div class="thinking-control"><span>思考程度</span><select aria-label="模型思考程度"><option value="low">低</option><option value="high">高</option></select><button class="thinking-reset" title="恢复默认">↺</button></div>
  </body>`, {runScripts:'dangerously', pretendToBeVisual:true});
  dom.window.eval(source);
  return dom;
}

test('图标按钮保留名称，下拉选择触发已有 change 处理', () => {
  const dom=setup(), doc=dom.window.document;
  const icon=doc.querySelector('#sidebar-toggle');
  assert.equal(icon.getAttribute('aria-label'),'收起侧边栏');
  assert.ok(icon.querySelector('svg[aria-hidden="true"]'));
  const select=doc.querySelector('#theme-select'), trigger=select.parentElement.querySelector('.choice-trigger');
  let changed=0;select.addEventListener('change',()=>changed++);
  trigger.click();const list=select.parentElement.querySelector('.choice-list');
  assert.equal(trigger.getAttribute('aria-expanded'),'true');
  list.querySelectorAll('.choice-option')[1].click();
  assert.equal(select.value,'mist');assert.equal(changed,1);
  assert.equal(trigger.textContent,'雾蓝浅色');assert.equal(trigger.getAttribute('aria-expanded'),'false');
  dom.window.close();
});

test('下拉支持方向键、Escape 和点击外部关闭', () => {
  const dom=setup(), doc=dom.window.document, select=doc.querySelector('#theme-select');
  const trigger=select.parentElement.querySelector('.choice-trigger');trigger.click();
  const list=select.parentElement.querySelector('.choice-list');
  list.dispatchEvent(new dom.window.KeyboardEvent('keydown',{key:'ArrowDown',bubbles:true}));
  assert.equal(doc.activeElement.textContent,'雾蓝浅色');
  list.dispatchEvent(new dom.window.KeyboardEvent('keydown',{key:'Escape',bubbles:true}));
  assert.equal(trigger.getAttribute('aria-expanded'),'false');assert.equal(doc.activeElement,trigger);
  trigger.click();doc.body.dispatchEvent(new dom.window.Event('pointerdown',{bubbles:true}));
  assert.equal(trigger.getAttribute('aria-expanded'),'false');dom.window.close();
});

test('加载期间禁用按钮，并同步模型思考程度', async () => {
  const dom=setup(), doc=dom.window.document, select=doc.querySelector('.thinking-control select');
  const trigger=select.parentElement.querySelector('.choice-trigger');
  select.disabled=true;select.value='high';
  await new Promise(r=>setTimeout(r,60));
  assert.equal(trigger.disabled,true);assert.equal(trigger.textContent,'高');
  assert.equal(doc.querySelector('.thinking-reset').getAttribute('title'),'恢复默认');
  dom.window.close();
});
