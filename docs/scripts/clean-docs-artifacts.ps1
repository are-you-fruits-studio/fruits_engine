param(
    [switch]$RemoveNodeModules
)

$ErrorActionPreference = "Stop"

function Remove-PathIfExists {
    param(
        [Parameter(Mandatory = $true)]
        [string]$Path
    )

    if (Test-Path -LiteralPath $Path) {
        Remove-Item -LiteralPath $Path -Recurse -Force
        Write-Host "Removed: $Path"
    }
}

$repoRoot = Resolve-Path -LiteralPath (Join-Path $PSScriptRoot "..\..")
$docsRoot = Join-Path $repoRoot "docs"

Set-Location $repoRoot

# Artifacts produced by the docs pipeline:
#   target/doc       - native rustdoc HTML (cargo doc)
#   docs/build       - Docusaurus static site with rustdoc copied under api-reference/
#   docs/.docusaurus - Docusaurus build cache
$artifacts = @(
    (Join-Path $repoRoot "target\doc"),
    (Join-Path $docsRoot "build"),
    (Join-Path $docsRoot ".docusaurus")
)

Write-Host "==> Cleaning generated docs artifacts"
foreach ($artifact in $artifacts) {
    Remove-PathIfExists $artifact
}

if ($RemoveNodeModules) {
    Write-Host "==> Cleaning npm dependencies"
    Remove-PathIfExists (Join-Path $docsRoot "node_modules")
}

Write-Host "==> Docs artifacts cleaned"
