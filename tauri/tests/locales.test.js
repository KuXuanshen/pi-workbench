'use strict';
const {test}=require('node:test');
const assert=require('node:assert/strict');
const fs=require('node:fs');
const path=require('node:path');

test('中英文文案键与占位符一致',()=>{
  const folder=path.join(__dirname,'../assets/locales');
  const zh=JSON.parse(fs.readFileSync(path.join(folder,'zh-CN.json'),'utf8'));
  const en=JSON.parse(fs.readFileSync(path.join(folder,'en.json'),'utf8'));
  assert.deepEqual(Object.keys(zh).sort(),Object.keys(en).sort());
  for(const key of Object.keys(zh)){
    const tokens=value=>[...new Set(value.match(/\{\w+\}/g)||[])].sort();
    assert.deepEqual(tokens(zh[key]),tokens(en[key]),key);
  }
});
