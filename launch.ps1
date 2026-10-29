$project = $PSScriptRoot
$electron = Join-Path $project 'node_modules\electron\dist\electron.exe'
if (-not (Test-Path -LiteralPath $electron)) { throw '请先运行 npm install。' }
& $electron (Join-Path $project 'desktop.js')
