param(
    [string]$Version = "",
    [string]$Root = (Resolve-Path (Join-Path $PSScriptRoot "..")).Path,
    [switch]$SkipBuild
)

$ErrorActionPreference = "Stop"

function Get-Sha256Hex {
    param([string]$Path)

    $stream = [System.IO.File]::OpenRead($Path)
    try {
        $sha256Algorithm = [System.Security.Cryptography.SHA256]::Create()
        try {
            $hashBytes = $sha256Algorithm.ComputeHash($stream)
            return -join ($hashBytes | ForEach-Object { $_.ToString("x2") })
        }
        finally {
            $sha256Algorithm.Dispose()
        }
    }
    finally {
        $stream.Dispose()
    }
}

$Root = (Resolve-Path -LiteralPath $Root).Path

if (-not $Version) {
    $packageJson = Get-Content -LiteralPath (Join-Path $Root "package.json") -Raw | ConvertFrom-Json
    $Version = [string]$packageJson.version
}

$cargoBin = Join-Path $env:USERPROFILE ".cargo\bin"
if ((Test-Path -LiteralPath $cargoBin -PathType Container) -and -not ($env:Path -split ';' | Where-Object { $_ -eq $cargoBin })) {
    $env:Path = "$cargoBin;$env:Path"
}

if (-not $SkipBuild) {
    Push-Location $Root
    try {
        & npm run desktop:build
        if ($LASTEXITCODE -ne 0) {
            throw "npm run desktop:build failed with exit code $LASTEXITCODE"
        }
    }
    finally {
        Pop-Location
    }
}

$releaseOutputRoot = Join-Path $Root "output\release"
$releaseRoot = Join-Path $releaseOutputRoot "time-state-recorder-desktop-v$Version"
$bundleRoot = Join-Path $Root "target\x86_64-pc-windows-gnullvm\release\bundle\nsis"

# Installer file pattern: Time State Recorder Desktop_$Version_x64-setup.exe
$tauriInstallerName = "Time State Recorder Desktop_${Version}_x64-setup.exe"
$installerName = "time-state-recorder-desktop-v$Version-windows-x64-setup.exe"
$installerPath = Join-Path $bundleRoot $tauriInstallerName
$releaseNotesSource = Join-Path $Root "docs\releases\v$Version.md"

if (-not (Test-Path -LiteralPath $installerPath -PathType Leaf)) {
    throw "Missing Tauri installer: $installerPath"
}
if (-not (Test-Path -LiteralPath $releaseNotesSource -PathType Leaf)) {
    throw "Missing release notes: $releaseNotesSource"
}

New-Item -ItemType Directory -Force -Path $releaseOutputRoot | Out-Null

$resolvedOutputRoot = (Resolve-Path -LiteralPath $releaseOutputRoot).Path
$candidateReleaseRoot = [System.IO.Path]::GetFullPath($releaseRoot)
if (-not $candidateReleaseRoot.StartsWith($resolvedOutputRoot, [System.StringComparison]::OrdinalIgnoreCase)) {
    throw "Refusing to clean release directory outside output/release: $candidateReleaseRoot"
}

if (Test-Path -LiteralPath $releaseRoot) {
    Remove-Item -LiteralPath $releaseRoot -Recurse -Force
}
New-Item -ItemType Directory -Force -Path $releaseRoot | Out-Null

$releaseInstallerPath = Join-Path $releaseRoot $installerName
$releaseNotesPath = Join-Path $releaseRoot "RELEASE_NOTES.md"
$sha256Path = "$releaseInstallerPath.sha256"
$manifestPath = Join-Path $releaseRoot "release-manifest.json"

Copy-Item -LiteralPath $installerPath -Destination $releaseInstallerPath
Copy-Item -LiteralPath $releaseNotesSource -Destination $releaseNotesPath

$sha256 = Get-Sha256Hex -Path $releaseInstallerPath
"$sha256  $installerName" | Set-Content -LiteralPath $sha256Path -Encoding ASCII

$sourceCommit = $null
try {
    Push-Location $Root
    $sourceCommit = (& git rev-parse HEAD).Trim()
}
finally {
    Pop-Location
}

$manifest = [ordered]@{
    productName = "Time State Recorder Desktop"
    version = $Version
    generatedAtUtc = (Get-Date).ToUniversalTime().ToString("o")
    sourceCommit = $sourceCommit
    tauriArtifactName = $tauriInstallerName
    artifactName = $installerName
    artifactPath = $releaseInstallerPath
    sha256 = $sha256
    sha256Path = $sha256Path
    releaseNotesPath = $releaseNotesPath
}

$manifest | ConvertTo-Json -Depth 4 | Set-Content -LiteralPath $manifestPath -Encoding ASCII

Write-Host "Packaged Time State Recorder Desktop v$Version"
Write-Host "Installer: $releaseInstallerPath"
Write-Host "SHA256: $sha256"
Write-Host "Manifest: $manifestPath"

Get-Item -LiteralPath $manifestPath
