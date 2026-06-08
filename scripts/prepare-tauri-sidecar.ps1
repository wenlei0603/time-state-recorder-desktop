$ErrorActionPreference = "Stop"

$repoRoot = Resolve-Path (Join-Path $PSScriptRoot "..")
$srcTauriDir = Join-Path $repoRoot "src-tauri"
$sidecarDir = Join-Path $srcTauriDir "bin"

function Get-TargetTriple {
  $hostTuple = (& rustc --print host-tuple 2>$null).Trim()
  if ($hostTuple) {
    return $hostTuple
  }

  $versionDetails = & rustc -Vv
  $hostLine = $versionDetails | Where-Object { $_ -like "host:*" } | Select-Object -First 1
  if (-not $hostLine) {
    throw "Unable to determine Rust target triple from rustc."
  }
  return ($hostLine -split "\s+")[1]
}

Push-Location $repoRoot
try {
  $targetTriple = Get-TargetTriple
  $isWindows = $env:OS -eq "Windows_NT"
  $extension = if ($isWindows) { ".exe" } else { "" }
  $targetFileName = if ($isWindows) {
    "tsr-collector-$targetTriple.exe"
  } else {
    "tsr-collector-$targetTriple"
  }
  $targetFileNames = @($targetFileName)
  if ($isWindows -and $targetTriple -eq "x86_64-pc-windows-gnullvm") {
    # The executable is gnullvm-built, but Tauri's Windows NSIS bundler asks for
    # the msvc target-suffixed externalBin path when assembling resources.
    $targetFileNames += "tsr-collector-x86_64-pc-windows-msvc.exe"
  }

  cargo build -p tsr-collector --release

  $sourceCandidates = @(
    (Join-Path $repoRoot "target\$targetTriple\release\tsr-collector$extension"),
    (Join-Path $repoRoot "target\release\tsr-collector$extension")
  )
  $source = $sourceCandidates | Where-Object { Test-Path $_ } | Select-Object -First 1
  if (-not $source) {
    throw "Unable to find tsr-collector release binary. Checked: $($sourceCandidates -join ', ')"
  }

  New-Item -ItemType Directory -Force -Path $sidecarDir | Out-Null
  foreach ($fileName in $targetFileNames) {
    $destination = Join-Path $sidecarDir $fileName
    Copy-Item -LiteralPath $source -Destination $destination -Force
    Write-Host "Prepared Tauri sidecar: $destination"
  }
} finally {
  Pop-Location
}
