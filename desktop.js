const {app,BrowserWindow,ipcMain,dialog}=require('electron');
const {spawn}=require('child_process');const path=require('path');let server,win;
app.setName('Pi Workbench');
app.whenReady().then(()=>{
  server=spawn(process.env.PI_WORKBENCH_NODE||'node',[path.join(__dirname,'server.js')],{cwd:__dirname,windowsHide:true,stdio:['ignore','pipe','pipe']});
  let buffer='',errors='';server.stderr.on('data',data=>{errors+=data.toString().slice(0,2000)});
  server.on('error',()=>{dialog.showErrorBox('Pi 工作台','无法启动 Node.js。请安装 Node.js 并确保 node 命令在 PATH 中。');app.quit()});
  server.stdout.on('data',data=>{buffer+=data.toString();let match=buffer.match(/http:\/\/127\.0\.0\.1:\d+\/#\w+/);if(!match||win)return;
    const options={width:1400,height:900,minWidth:620,minHeight:420,frame:false,backgroundColor:'#111b2a',show:false,webPreferences:{preload:path.join(__dirname,'preload.js'),contextIsolation:true,nodeIntegration:false,sandbox:true}};
    if(process.platform==='win32')options.icon=path.join(__dirname,'workbench.ico');
    win=new BrowserWindow(options);win.once('ready-to-show',()=>win.show());win.loadURL(match[0]);win.webContents.setWindowOpenHandler(()=>({action:'deny'}));
    const allowedOrigin=match[0].split('/#')[0];win.webContents.on('will-navigate',(event,url)=>{if(!url.startsWith(allowedOrigin+'/'))event.preventDefault()});win.on('closed',()=>{win=null;app.quit()});
  });
  server.on('exit',(code)=>{if(!win){if(code)dialog.showErrorBox('Pi 工作台','本地服务启动失败，请检查 Node.js、依赖和 Pi 配置。');app.quit()}});
});
ipcMain.on('window-action',(e,action)=>{if(!win||e.sender!==win.webContents)return;if(action==='paste')win.webContents.paste();if(action==='minimize')win.minimize();if(action==='maximize')win.isMaximized()?win.unmaximize():win.maximize();if(action==='close')win.close()});
app.on('before-quit',()=>{if(server&&!server.killed)server.kill()});app.on('window-all-closed',()=>app.quit());
