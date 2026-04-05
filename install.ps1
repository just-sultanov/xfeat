param(
  [string]$Prefix = "$env:USERPROFILE\.local\bin",
  [string]$Version = ""
)

$ErrorActionPreference = "Stop"

$Arch = (Get-CimInstance Win32_Processor).Architecture
if ($Arch -eq 9) { $Target = "x86_64-pc-windows-gnu" }
else { Write-Error "Unsupported architecture"; exit 1 }

if (-not $Version) {
  $Release = Invoke-RestMethod "https://api.github.com/repos/just-sultanov/xfeat/releases/latest"
  $Version = $Release.tag_name -replace 'v', ''
}

$Url = "https://github.com/just-sultanov/xfeat/releases/download/v${Version}/xfeat-${Target}.zip"

$Tmp = Join-Path $env:TEMP "xfeat-install"
New-Item -ItemType Directory -Force -Path $Tmp | Out-Null
Invoke-WebRequest -Uri $Url -OutFile "$Tmp\xfeat.zip"
Expand-Archive -Path "$Tmp\xfeat.zip" -DestinationPath $Tmp -Force

New-Item -ItemType Directory -Force -Path $Prefix | Out-Null
Move-Item "$Tmp\xfeat.exe" "$Prefix\xfeat.exe" -Force
Remove-Item $Tmp -Recurse -Force

Write-Host "xfeat v${Version} installed to ${Prefix}\xfeat.exe"
