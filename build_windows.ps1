$ErrorActionPreference = "Stop"

$AppName = "NaviTune"
$BinName = "navitune.exe"
$Version = (Select-String -Path "Cargo.toml" -Pattern '^version\s*=\s*"([^"]+)"' | Select-Object -First 1).Matches.Groups[1].Value
$DistDir = "dist"
$StageDir = Join-Path $DistDir "$AppName-$Version-Windows-x86_64"

$HostTarget = ((rustc -vV | Select-String "^host:").ToString().Split(":")[1]).Trim()
if (-not $HostTarget) { $HostTarget = "x86_64-pc-windows-gnu" }

Write-Host "==> Building $BinName for Windows ($HostTarget)..."
cargo build --release --target $HostTarget

if (Test-Path $StageDir) { Remove-Item -Recurse -Force $StageDir }
New-Item -ItemType Directory -Force -Path $StageDir | Out-Null

Copy-Item "target\$HostTarget\release\$BinName" -Destination (Join-Path $StageDir $BinName)
if (Test-Path "README.md") { Copy-Item "README.md" -Destination $StageDir }

$ZipPath = Join-Path $DistDir "$AppName-$Version-Windows-x86_64.zip"
if (Test-Path $ZipPath) { Remove-Item -Force $ZipPath }
Compress-Archive -Path "$StageDir\*" -DestinationPath $ZipPath

Write-Host "==> Done! Output artifacts:"
Get-ChildItem $ZipPath
