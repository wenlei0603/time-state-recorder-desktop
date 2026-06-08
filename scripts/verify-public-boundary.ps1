param(
    [string]$Root = (Resolve-Path (Join-Path $PSScriptRoot "..")).Path
)

$ErrorActionPreference = "Stop"

Push-Location $Root
try {
    $tracked = git ls-files
    $blockedPathPattern = '(^|/)(data|logs|\.launcher|\.worktrees|\.claude|\.playwright-mcp|\.superpowers)(/|$)|\.env\.local$|\.sqlite3$|\.db$|\.db-wal$|\.db-shm$'
    $blocked = $tracked | Select-String -Pattern $blockedPathPattern
    if ($blocked) {
        Write-Error "Blocked private/runtime paths are tracked:`n$($blocked -join [Environment]::NewLine)"
    }

    $secretHits = git grep -n -I -E 'api[_-]?key|bearer |sk-[A-Za-z0-9]|MINIMAX_API_KEY|OPENAI_API_KEY' -- . 2>$null
    $allowedSecretHitPattern = 'examples/config/config\.example\.json|SECURITY\.md|CONTRIBUTING\.md|docs/product/|docs/security/public-repo-boundary\.md|scripts/verify-public-boundary\.ps1|README\.md'
    $unexpectedSecretHits = $secretHits | Where-Object { $_ -notmatch $allowedSecretHitPattern }
    if ($unexpectedSecretHits) {
        Write-Error "Unexpected secret-like strings found:`n$($unexpectedSecretHits -join [Environment]::NewLine)"
    }

    Write-Host "Public boundary verification passed."
} finally {
    Pop-Location
}
