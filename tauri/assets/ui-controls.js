'use strict';
// Small, local icon set: paths are drawn for this application and inherit the theme colour.
const iconPaths = {
  menu:'M4 7h16M4 12h16M4 17h16', settings:'M9.64 4.36L10.21 2.16L13.79 2.16L14.36 4.36L15.73 4.92L17.69 3.78L20.22 6.31L19.08 8.27L19.64 9.64L21.84 10.21L21.84 13.79L19.64 14.36L19.08 15.73L20.22 17.69L17.69 20.22L15.73 19.08L14.36 19.64L13.79 21.84L10.21 21.84L9.64 19.64L8.27 19.08L6.31 20.22L3.78 17.69L4.92 15.73L4.36 14.36L2.16 13.79L2.16 10.21L4.36 9.64L4.92 8.27L3.78 6.31L6.31 3.78L8.27 4.92Z M15 12a3 3 0 1 0 -6 0a3 3 0 1 0 6 0Z',
  fullscreen:'M4 9V4h5M15 4h5v5M20 15v5h-5M9 20H4v-5', refresh:'M20 7v5h-5M4 17v-5h5M5.4 9a7 7 0 0 1 11.5-2L20 12M4 12l3.1 5a7 7 0 0 0 11.5-2',
  plus:'M12 5v14M5 12h14', reset:'M3 12a9 9 0 1 0 2.6-6.4L3 8M3 3v5h5',
  grip:'M9 5h.01M15 5h.01M9 12h.01M15 12h.01M9 19h.01M15 19h.01',
  minimize:'M5 12h14', edit:'M4 20l4.5-1 10-10-3.5-3.5-10 10L4 20zM13.5 7l3.5 3.5', close:'M6 6l12 12M18 6 6 18',
  maximize:'M5 5h14v14H5z', terminal:'M4 6l6 6-6 6M13 18h7', restore:'M7 17 17 7M8 7h9v9', chevron:'M7 10l5 5 5-5'
};
function makeIcon(name) {
  const svg=document.createElementNS('http://www.w3.org/2000/svg','svg');
  svg.setAttribute('viewBox','0 0 24 24');svg.setAttribute('aria-hidden','true');svg.setAttribute('focusable','false');svg.setAttribute('class','ui-icon');
  const path=document.createElementNS('http://www.w3.org/2000/svg','path');path.setAttribute('d',iconPaths[name]);svg.append(path);return svg;
}
function useIcon(element,name){if(!element||element.dataset.uiIcon===name)return;element.dataset.uiIcon=name;element.replaceChildren(makeIcon(name));}
function refreshIcons(){
  for(const [selector,name] of Object.entries({'#sidebar-toggle':'menu','#settings-toggle':'settings','#fullscreen':'fullscreen','#refresh':'refresh','#win-min':'minimize','#win-max':'maximize','#win-close':'close','.empty-icon':'terminal'}))useIcon(document.querySelector(selector),name);
  document.querySelectorAll('.model-create').forEach(e=>useIcon(e,'plus'));
  document.querySelectorAll('.thinking-reset').forEach(e=>useIcon(e,'reset'));
  document.querySelectorAll('.pane-head').forEach(head=>{const buttons=head.querySelectorAll('button');['grip','minimize','edit','close'].forEach((name,i)=>useIcon(buttons[i],name))});
  document.querySelectorAll('.dock-restore').forEach(e=>useIcon(e,'restore'));
}

// The original select remains the source of truth, including its existing change handlers.
let openedControl=null;
function closeSelect(restoreFocus=false){if(!openedControl)return;const control=openedControl;openedControl=null;control.list.hidden=true;control.button.setAttribute('aria-expanded','false');control.wrap.classList.remove('is-open');if(restoreFocus)control.button.focus()}
function syncSelect(control){
  const {select,button,display}=control, selected=select.selectedOptions[0];
  const text=selected?.textContent||'', label=select.getAttribute('aria-label')||select.labels?.[0]?.textContent.trim()||'';
  if(display.textContent!==text)display.textContent=text;if(button.disabled!==select.disabled)button.disabled=select.disabled;
  if(button.getAttribute('aria-label')!==label)button.setAttribute('aria-label',label);
  if(button.title!==(select.title||''))button.title=select.title||'';if(select.disabled && openedControl===control)closeSelect();
}
function placeList(control){
  const rect=control.button.getBoundingClientRect(), list=control.list;
  const height=Math.min(list.scrollHeight||240,240), spaceBelow=window.innerHeight-rect.bottom;
  list.style.left=Math.max(8,Math.min(rect.left,window.innerWidth-Math.max(rect.width,160)-8))+'px';
  list.style.width=Math.max(rect.width,160)+'px';
  list.style.top=(spaceBelow<height+8&&rect.top>spaceBelow?Math.max(8,rect.top-height-6):rect.bottom+6)+'px';
}
function openSelect(control){
  if(control.button.disabled)return;if(openedControl===control){closeSelect();return}closeSelect();openedControl=control;
  const {select,list,button}=control;list.replaceChildren();
  for(const option of select.options){if(option.hidden)continue;const item=document.createElement('button');item.type='button';item.className='choice-option';item.setAttribute('role','option');item.setAttribute('aria-selected',String(option.selected));item.disabled=option.disabled;item.textContent=option.textContent;item.addEventListener('click',()=>{select.value=option.value;select.dispatchEvent(new Event('change',{bubbles:true}));syncSelect(control);closeSelect(true)});list.append(item)}
  list.hidden=false;button.setAttribute('aria-expanded','true');control.wrap.classList.add('is-open');placeList(control);
  (list.querySelector('[aria-selected="true"]:not(:disabled)')||list.querySelector('button:not(:disabled)'))?.focus();
}
function upgradeSelect(select){
  if(select.dataset.enhanced||select.classList.contains('first-language'))return;
  select.dataset.enhanced='true';const wrap=document.createElement('span');wrap.className='choice-wrap';select.before(wrap);wrap.append(select);select.classList.add('choice-native');select.tabIndex=-1;select.setAttribute('aria-hidden','true');
  const button=document.createElement('button'),display=document.createElement('span'),list=document.createElement('span');button.type='button';button.className='choice-trigger';button.setAttribute('role','combobox');button.setAttribute('aria-haspopup','listbox');button.setAttribute('aria-expanded','false');display.className='choice-value';button.append(display,makeIcon('chevron'));button.lastChild.classList.add('choice-chevron');list.className='choice-list';list.setAttribute('role','listbox');list.id='choice-'+Math.random().toString(36).slice(2);list.hidden=true;button.setAttribute('aria-controls',list.id);wrap.append(button,list);
  const control={wrap,select,button,display,list};select._choiceControl=control;button.addEventListener('click',()=>openSelect(control));select.addEventListener('change',()=>syncSelect(control));
  button.addEventListener('keydown',event=>{if(['ArrowDown','ArrowUp','Enter',' '].includes(event.key)){event.preventDefault();openSelect(control)}});
  list.addEventListener('keydown',event=>{const items=[...list.querySelectorAll('button:not(:disabled)')],index=items.indexOf(document.activeElement);if(event.key==='Escape'){event.preventDefault();closeSelect(true)}else if(event.key==='ArrowDown'||event.key==='ArrowUp'){event.preventDefault();items[(index+(event.key==='ArrowDown'?1:-1)+items.length)%items.length]?.focus()}else if(event.key==='Home'||event.key==='End'){event.preventDefault();items[event.key==='Home'?0:items.length-1]?.focus()}else if(event.key==='Tab'){closeSelect()}});
  syncSelect(control);
}
function updateControls(){refreshIcons();document.querySelectorAll('select').forEach(select=>{upgradeSelect(select);if(select._choiceControl)syncSelect(select._choiceControl)})}
document.addEventListener('pointerdown',event=>{if(openedControl&&!openedControl.wrap.contains(event.target))closeSelect()});
window.addEventListener('resize',()=>{if(openedControl)placeList(openedControl)});
document.addEventListener('scroll',()=>{if(openedControl)placeList(openedControl)},true);
let updatePending=false;
new MutationObserver(records=>{if(updatePending)return;const onlyTerminal=records.every(record=>{const el=record.target.nodeType===1?record.target:record.target.parentElement;return el?.closest('.term, .dock-preview')});if(onlyTerminal)return;updatePending=true;requestAnimationFrame(()=>{updatePending=false;updateControls()})}).observe(document.body,{subtree:true,childList:true,attributes:true,attributeFilter:['disabled','aria-label','title']});
updateControls();
