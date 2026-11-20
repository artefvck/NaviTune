$ErrorActionPreference = "Stop"

$AppName = "NaviTune"
$BinName = "pptool.exe"
$Version = (Select-String -Path "Cargo.toml" -Pattern '^version\s*=\s*"([^"]+)"' | Select-Object -First 1).Matches.Groups[1].Value
$DistDir = "dist"
$StageDir = Join-Path $DistDir "$AppName-$Version-Windows-x86_64"

Write-Host "==> Building $BinName for Windows (x86_64-pc-windows-msvc)..."
cargo build --release --target x86_64-pc-windows-msvc

if (Test-Path $StageDir) { Remove-Item -Recurse -Force $StageDir }
New-Item -ItemType Directory -Force -Path $StageDir | Out-Null

Copy-Item "target\x86_64-pc-windows-msvc\release\$BinName" -Destination (Join-Path $StageDir $BinName)
if (Test-Path "README.md") { Copy-Item "README.md" -Destination $StageDir }

$ZipPath = Join-Path $DistDir "$AppName-$Version-Windows-x86_64.zip"
if (Test-Path $ZipPath) { Remove-Item -Force $ZipPath }
Compress-Archive -Path "$StageDir\*" -DestinationPath $ZipPath

Write-Host "==> Done! Output artifacts:"
Get-ChildItem $ZipPath
