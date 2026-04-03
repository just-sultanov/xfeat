New-Item -ItemType Directory -Force -Path dist
$v = (./target/x86_64-pc-windows-gnu/release/xfeat.exe --version).Split(' ')[1]
Compress-Archive -Path 'target\x86_64-pc-windows-gnu\release\xfeat.exe' `
  -DestinationPath "dist\xfeat-v$v-x86_64-pc-windows-gnu.zip"
