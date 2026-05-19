$ErrorActionPreference = "Stop"

function Invoke-Native {
    param(
        [Parameter(Mandatory = $true)]
        [string]$FilePath,
        [string[]]$ArgumentList = @()
    )

    & $FilePath @ArgumentList
    if ($LASTEXITCODE -ne 0) {
        throw "Command failed with exit code ${LASTEXITCODE}: $FilePath $($ArgumentList -join ' ')"
    }
}

$repoRoot = Resolve-Path -LiteralPath (Join-Path $PSScriptRoot "..\..")
$apiDir = Join-Path $repoRoot "docs\api-reference"
$basePath = "/api-reference"

Set-Location $repoRoot

if ($env:SKIP_RUSTDOC_JSON -ne "1") {
    $env:RUSTDOCFLAGS = if ($env:RUSTDOCFLAGS) {
        $env:RUSTDOCFLAGS
    } else {
        "-Z unstable-options --output-format json"
    }

    Invoke-Native "cargo" @("+nightly", "doc", "--workspace", "--no-deps")
}

Get-ChildItem -LiteralPath $apiDir -Force |
    Where-Object { $_.Name -ne ".gitkeep" } |
    ForEach-Object { Remove-Item -LiteralPath $_.FullName -Recurse -Force }

New-Item -ItemType Directory -Force -Path $apiDir | Out-Null

Invoke-Native "cargo" @("doc-docusaurus", "components", "init", "docs")

$jsonFiles = Get-ChildItem -LiteralPath (Join-Path $repoRoot "target\doc") -Filter "*.json" |
    Sort-Object BaseName

if (-not $jsonFiles) {
    throw "No rustdoc JSON files found in target\doc. Run rustdoc JSON generation first."
}

$workspaceCrates = ($jsonFiles | ForEach-Object { $_.BaseName }) -join ","

foreach ($json in $jsonFiles) {
    Invoke-Native "cargo" @(
        "doc-docusaurus",
        $json.FullName,
        "-o",
        $apiDir,
        "--base-path",
        $basePath,
        "--workspace-crates",
        $workspaceCrates
    )
}

Get-ChildItem -LiteralPath $apiDir -Recurse |
    Where-Object { -not $_.PSIsContainer -and ($_.Extension -eq ".md" -or $_.Extension -eq ".mdx") } |
    ForEach-Object {
    $text = Get-Content -LiteralPath $_.FullName -Raw
    $text = [regex]::Replace($text, "^displayed_sidebar:.*\r?\n", "", [System.Text.RegularExpressions.RegexOptions]::Multiline)
    $text = [regex]::Replace($text, "(?<!!)\[([^\]\r\n]+)\]\((?!https?://|mailto:|#)([^)\r\n]+)\)", '$1')
    $text = [regex]::Replace($text, '<Link\b(?=[^>]*\bto="(?!https?://|mailto:|#)[^"]*")[^>]*>(.*?)</Link>', '$1', [System.Text.RegularExpressions.RegexOptions]::Singleline)
    $text = [regex]::Replace($text, "^import Link from .@docusaurus/Link.;\r?\n", "", [System.Text.RegularExpressions.RegexOptions]::Multiline)
    Set-Content -LiteralPath $_.FullName -Value $text -NoNewline
}
