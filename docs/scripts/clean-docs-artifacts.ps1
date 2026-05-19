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
$apiReferenceDir = Join-Path $docsRoot "api-reference"
$targetDocDir = Join-Path $repoRoot "target\doc"

Set-Location $repoRoot

Write-Host "==> Cleaning generated API reference markdown"
if (Test-Path -LiteralPath $apiReferenceDir) {
    Get-ChildItem -LiteralPath $apiReferenceDir -Force |
        Where-Object { $_.Name -ne ".gitkeep" } |
        ForEach-Object {
            Remove-Item -LiteralPath $_.FullName -Recurse -Force
            Write-Host "Removed: $($_.FullName)"
        }
}

Write-Host "==> Cleaning rustdoc JSON files"
if (Test-Path -LiteralPath $targetDocDir) {
    Get-ChildItem -LiteralPath $targetDocDir -Filter "*.json" -File |
        ForEach-Object {
            Remove-Item -LiteralPath $_.FullName -Force
            Write-Host "Removed: $($_.FullName)"
        }
}

Write-Host "==> Cleaning Docusaurus build artifacts"
Remove-PathIfExists (Join-Path $docsRoot "build")
Remove-PathIfExists (Join-Path $docsRoot ".docusaurus")

Write-Host "==> Cleaning cargo-doc-docusaurus generated files"
Remove-PathIfExists (Join-Path $docsRoot "sidebars-rust.ts")
Remove-PathIfExists (Join-Path $docsRoot "src\components\RustCode")
Remove-PathIfExists (Join-Path $docsRoot "src\components\RustCrateLink")
Remove-PathIfExists (Join-Path $docsRoot "src\components\RustModuleTitle")
Remove-PathIfExists (Join-Path $docsRoot "src\theme\DocSidebarItem")
Remove-PathIfExists (Join-Path $docsRoot "src\css\rust-documentation.css")

if ($RemoveNodeModules) {
    Write-Host "==> Cleaning npm dependencies"
    Remove-PathIfExists (Join-Path $docsRoot "node_modules")
}

Write-Host "==> Docs artifacts cleaned"
